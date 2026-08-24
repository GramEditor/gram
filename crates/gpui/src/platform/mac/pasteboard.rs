use itertools::Itertools;
use objc2::{ClassType, rc::Retained};
use objc2_app_kit::{
    NSPasteboard, NSPasteboardItem, NSPasteboardName, NSPasteboardNameFind, NSPasteboardType, NSPasteboardTypePNG,
    NSPasteboardTypeString, NSPasteboardTypeTIFF,
};
use objc2_foundation::{NSArray, NSData, NSString, NSURL, ns_string};
use smallvec::SmallVec;
use std::path::PathBuf;
use strum::IntoEnumIterator as _;

use crate::{ClipboardEntry, ClipboardItem, ClipboardString, Image, ImageFormat, asset_cache::hash};

static TEXT_HASH_TYPE: &'static str = "gram-text-hash";
static METADATA_TYPE: &'static str = "gram-metadata";

pub struct Pasteboard {
    inner: Retained<NSPasteboard>,
}

struct SafeNS {}

// SAFETY: Most objc2 extern statics are safe to use, the lib is just stuck on a rust MSRV without
// 'static safe' support:
// https://github.com/madsmtm/objc2/issues/255
impl SafeNS {
    fn type_string() -> &'static NSPasteboardType {
        unsafe { NSPasteboardTypeString }
    }

    fn type_png() -> &'static NSPasteboardType {
        unsafe { NSPasteboardTypePNG }
    }

    fn type_tiff() -> &'static NSPasteboardType {
        unsafe { NSPasteboardTypeTIFF }
    }

    fn name_find() -> &'static NSPasteboardName {
        unsafe { NSPasteboardNameFind }
    }
}

impl Pasteboard {
    pub fn general() -> Self {
        Self::new(NSPasteboard::generalPasteboard())
    }

    pub fn find() -> Self {
        Self::new(NSPasteboard::pasteboardWithName(SafeNS::name_find()))
    }

    #[cfg(test)]
    pub fn unique() -> Self {
        Self::new(NSPasteboard::pasteboardWithUniqueName())
    }

    fn new(inner: Retained<NSPasteboard>) -> Self {
        Self { inner }
    }

    pub fn read(&self) -> Option<ClipboardItem> {
        let class_array = NSArray::from_slice(&[
            // File copies from outside the system
            NSURL::class(),
            // Text data
            NSString::class(),
            // Image/Raw Data
            NSPasteboardItem::class(),
        ]);
        // SAFETY: All classes passed here are supported by 'readObjectsForClasses'
        // https://developer.apple.com/documentation/appkit/nspasteboard/readobjects(forclasses:options:)
        let objects = unsafe { self.inner.readObjectsForClasses_options(&class_array, None) }?;

        // Items are guaranteed to follow the order specified in 'class_array'. Thus, we need to
        // drain the iterator in the same order.
        let mut objects = objects.into_iter().peekable();

        // Drain NSURL items
        let mut url_paths = SmallVec::<[PathBuf; 2]>::new();
        while let Some(url) = objects.peek().and_then(|o| o.downcast_ref::<NSURL>()) {
            if url.isFileURL() {
                if let Some(path) = url.to_file_path() {
                    url_paths.push(path);
                }
            }
            objects.next();
        }
        if !url_paths.is_empty() {
            return Some(ClipboardItem {
                entries: vec![ClipboardEntry::ExternalPaths(crate::ExternalPaths(url_paths))],
            });
        }

        // Peek single item for NSString
        if let Some(s) = objects.peek().and_then(|o| o.downcast_ref::<NSString>()) {
            return Some(self.read_string(s));
        }

        // Drain NSPasteboardItem items (images)
        while let Some(item) = objects.peek().and_then(|o| o.downcast_ref::<NSPasteboardItem>()) {
            if let Some(image_item) = self.read_image(item) {
                return Some(image_item);
            }
            objects.next();
        }

        None
    }

    fn read_image(&self, item: &NSPasteboardItem) -> Option<ClipboardItem> {
        let (data, format) = ImageFormat::iter().find_map(|format| {
            let ut_type: UTType = format.into();
            item.dataForType(ut_type.inner()).zip(Some(format))
        })?;
        let bytes = data.to_vec();
        let id = hash(&bytes);
        return Some(ClipboardItem {
            entries: vec![ClipboardEntry::Image(Image { format, bytes, id })],
        });
    }

    fn read_string(&self, ns_text: &NSString) -> ClipboardItem {
        let text_hash_type = ns_string!(TEXT_HASH_TYPE);
        let metadata_type = ns_string!(METADATA_TYPE);
        let text = ns_text.to_string();
        let metadata = self.inner.dataForType(text_hash_type).and_then(|hash_data| {
            let hash_bytes = hash_data.to_vec().try_into().ok()?;
            let hash = u64::from_be_bytes(hash_bytes);
            let metadata_data = self.inner.dataForType(metadata_type)?;

            if hash == ClipboardString::text_hash(&text) {
                String::from_utf8(metadata_data.to_vec()).ok()
            } else {
                None
            }
        });

        ClipboardItem {
            entries: vec![ClipboardEntry::String(ClipboardString { text, metadata })],
        }
    }

    pub fn write(&self, item: ClipboardItem) {
        match item.entries.as_slice() {
            [] => {
                // Writing an empty list of entries just clears the clipboard.
                self.inner.clearContents();
            }
            [ClipboardEntry::String(string)] => {
                self.write_plaintext(string);
            }
            [ClipboardEntry::Image(image)] => {
                self.write_image(image);
            }
            [ClipboardEntry::ExternalPaths(paths)] => {
                // In practice we should never reach here, but we can handle it for completness sake.
                self.write_paths(paths);
            }
            _ => {
                // Agus NB: We're currently only writing string entries to the clipboard when we have more than one.
                //
                // This was the existing behavior before I refactored the outer clipboard code:
                // https://github.com/zed-industries/zed/blob/65f7412a0265552b06ce122655369d6cc7381dd6/crates/gpui/src/platform/mac/platform.rs#L1060-L1110
                //
                // Note how `any_images` is always `false`. We should fix that, but that's orthogonal to the refactor.

                let mut combined = ClipboardString {
                    text: String::new(),
                    metadata: None,
                };

                for entry in item.entries {
                    match entry {
                        ClipboardEntry::String(text) => {
                            combined.text.push_str(&text.text());
                            if combined.metadata.is_none() {
                                combined.metadata = text.metadata;
                            }
                        }
                        _ => {}
                    }
                }

                self.write_plaintext(&combined);
            }
        }
    }

    fn write_plaintext(&self, string: &ClipboardString) {
        let text_hash_type = ns_string!(TEXT_HASH_TYPE);
        let metadata_type = ns_string!(METADATA_TYPE);
        self.inner.clearContents();

        let string_data = NSString::from_str(&string.text);
        self.inner.setString_forType(&string_data, SafeNS::type_string());

        if let Some(metadata) = string.metadata.as_ref() {
            let hash_bytes = ClipboardString::text_hash(&string.text).to_be_bytes();
            let hash_data = &NSData::with_bytes(&hash_bytes);
            self.inner.setData_forType(Some(hash_data), text_hash_type);
            let metadata_data = NSString::from_str(&metadata);
            self.inner.setString_forType(&metadata_data, metadata_type);
        }
    }

    fn write_image(&self, image: &Image) {
        self.inner.clearContents();

        if image.bytes().len() > 0 {
            let image_type = Into::<UTType>::into(image.format).inner();
            let image_data = NSData::with_bytes(&image.bytes);
            self.inner.setData_forType(Some(&image_data), image_type);
        }
    }

    fn write_paths(&self, paths: &crate::ExternalPaths) {
        self.inner.clearContents();

        let text = paths.0.iter().map(|p| p.to_string_lossy()).join("\n");
        if text.len() > 0 {
            self.write_plaintext(&ClipboardString { text, metadata: None });
        }
    }
}

impl From<ImageFormat> for UTType {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::Png => Self::png(),
            ImageFormat::Jpeg => Self::jpeg(),
            ImageFormat::Tiff => Self::tiff(),
            ImageFormat::Webp => Self::webp(),
            ImageFormat::Gif => Self::gif(),
            ImageFormat::Bmp => Self::bmp(),
            ImageFormat::Svg => Self::svg(),
            ImageFormat::Ico => Self::ico(),
        }
    }
}

// See https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/
pub struct UTType(&'static NSPasteboardType);

impl UTType {
    pub fn png() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/png
        Self(SafeNS::type_png()) // This is a rare case where there's a built-in NSPasteboardType
    }

    pub fn jpeg() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/jpeg
        Self(ns_string!("public.jpeg"))
    }

    pub fn gif() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/gif
        Self(ns_string!("com.compuserve.gif"))
    }

    pub fn webp() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/webp
        Self(ns_string!("org.webmproject.webp"))
    }

    pub fn bmp() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/bmp
        Self(ns_string!("com.microsoft.bmp"))
    }

    pub fn svg() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/svg
        Self(ns_string!("public.svg-image"))
    }

    pub fn ico() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/ico
        Self(ns_string!("com.microsoft.ico"))
    }

    pub fn tiff() -> Self {
        // https://developer.apple.com/documentation/uniformtypeidentifiers/uttype-swift.struct/tiff
        Self(SafeNS::type_tiff()) // This is a rare case where there's a built-in NSPasteboardType
    }

    fn inner(&self) -> &'static NSPasteboardType {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use objc2_app_kit::NSPasteboardTypeFileURL;

    use crate::{ClipboardEntry, ClipboardItem, ClipboardString, MINIMAL_BMP};

    use super::*;

    impl SafeNS {
        fn type_file_url() -> &'static NSPasteboardType {
            unsafe { NSPasteboardTypeFileURL }
        }
    }

    #[test]
    fn test_string() {
        let pasteboard = Pasteboard::unique();
        assert_eq!(pasteboard.read(), None);

        let item = ClipboardItem::new_string("1".to_string());
        pasteboard.write(item.clone());
        assert_eq!(pasteboard.read(), Some(item));

        let item = ClipboardItem {
            entries: vec![ClipboardEntry::String(
                ClipboardString::new("2".to_string()).with_json_metadata(vec![3, 4]),
            )],
        };
        pasteboard.write(item.clone());
        assert_eq!(pasteboard.read(), Some(item));

        let text_from_other_app = "text from other app";
        let bytes_data = NSString::from_str(text_from_other_app);
        pasteboard.inner.setString_forType(&bytes_data, SafeNS::type_string());
        assert_eq!(
            pasteboard.read(),
            Some(ClipboardItem::new_string(text_from_other_app.to_string()))
        );
    }

    #[test]
    fn test_paths() {
        let pasteboard = Pasteboard::unique();
        assert_eq!(pasteboard.read(), None);

        let paths = [PathBuf::from("/home/ubuntu/a.txt"), PathBuf::from("/etc/hosts.txt")];
        let item = ClipboardItem::new_external_paths(&paths);
        pasteboard.write(item);
        let result = ClipboardItem::new_string("/home/ubuntu/a.txt\n/etc/hosts.txt".to_string());
        assert_eq!(pasteboard.read(), Some(result));

        let path_from_other_app = "/Users/gram/a.txt";
        let bytes_data = NSString::from_str(&format!("file://{}", path_from_other_app));
        pasteboard.inner.setString_forType(&bytes_data, SafeNS::type_file_url());
        assert_eq!(
            pasteboard.read(),
            Some(ClipboardItem::new_external_paths(&[PathBuf::from(path_from_other_app)]))
        );
    }

    #[test]
    fn test_image() {
        let pasteboard = Pasteboard::unique();
        assert_eq!(pasteboard.read(), None);

        let image_data: Vec<u8> = MINIMAL_BMP.to_vec();
        let bytes_data = NSData::from_vec(image_data.clone());
        let item = ClipboardItem::new_image(&Image::from_bytes(ImageFormat::Bmp, image_data));

        pasteboard.inner.setData_forType(Some(&bytes_data), UTType::bmp().0);
        assert_eq!(pasteboard.read(), Some(item.clone()));

        pasteboard.write(item.clone());
        assert_eq!(pasteboard.read(), Some(item));

        pasteboard.write(ClipboardItem::new_image(&Image::empty()));
        assert_eq!(pasteboard.read(), None);
    }
}
