use anyhow::Result;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use rpc::proto::Envelope;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct MessageId(pub u32);

pub type MessageLen = u32;
pub const MESSAGE_LEN_SIZE: usize = size_of::<MessageLen>();

// We start each message with a 4-byte magic value:
// {0x47, 0x52, 0x41, 0x4D}, or "GRAM" in ASCII.
// The goal is to always trigger a parse error if a protobuf-based version of Gram
// should ever talk to a Postcard-based version of Gram and vice versa.
// The bytes "GRAM" are actually suspiciously well-suited for this purpose.
// a Protobuf message's wire format is encoded as a tag, then length, then value.
// The tag is '(field_number << 3) | wire_type', meaning the bottom 3 bits are a wire type.
// The valid wire types are: 0=VARINT, 1=I64, 2=LEN, 3=SGROUP, 4=EGROUP, 5=I32.
// Values 6 and 7 are unused. The 3 low bits in 0x47 ('G') is 111, or 7.
// So by starting the magic byte with 'G', we're starting it with a byte
// which Protobuf will read as an invalid wire type.
const MAGIC: &[u8] = b"GRAM";

pub fn message_len_from_buffer(buffer: &[u8]) -> MessageLen {
    MessageLen::from_le_bytes(buffer.try_into().unwrap())
}

pub async fn read_message_with_len<S: AsyncRead + Unpin>(
    stream: &mut S,
    buffer: &mut Vec<u8>,
    message_len: MessageLen,
) -> Result<Envelope> {
    buffer.resize(message_len as usize, 0);
    stream.read_exact(buffer).await?;

    if buffer.len() < MAGIC.len() {
        anyhow::bail!("Received too short message");
    }
    if &buffer[..MAGIC.len()] != MAGIC {
        anyhow::bail!("Received message with invalid magic")
    }

    let payload = &buffer[MAGIC.len()..];
    Ok(postcard::from_bytes::<Envelope>(payload)?)
}

pub async fn read_message<S: AsyncRead + Unpin>(stream: &mut S, buffer: &mut Vec<u8>) -> Result<Envelope> {
    buffer.resize(MESSAGE_LEN_SIZE, 0);
    stream.read_exact(buffer).await?;

    let len = message_len_from_buffer(buffer);

    read_message_with_len(stream, buffer, len).await
}

pub async fn write_message<S: AsyncWrite + Unpin>(
    stream: &mut S,
    buffer: &mut Vec<u8>,
    message: Envelope,
) -> Result<()> {
    buffer.clear();
    buffer.extend(MAGIC);
    postcard::to_io(&message, &mut *buffer)?;
    let message_len = buffer.len() as u32;

    stream.write_all(message_len.to_le_bytes().as_slice()).await?;
    stream.write_all(buffer).await?;
    Ok(())
}

pub async fn write_size_prefixed_buffer<S: AsyncWrite + Unpin>(stream: &mut S, buffer: &mut [u8]) -> Result<()> {
    let len = buffer.len() as u32;
    stream.write_all(len.to_le_bytes().as_slice()).await?;
    stream.write_all(buffer).await?;
    Ok(())
}

pub async fn read_message_raw<S: AsyncRead + Unpin>(stream: &mut S, buffer: &mut Vec<u8>) -> Result<()> {
    buffer.resize(MESSAGE_LEN_SIZE, 0);
    stream.read_exact(buffer).await?;

    let message_len = message_len_from_buffer(buffer);
    buffer.resize(message_len as usize, 0);
    stream.read_exact(buffer).await?;

    Ok(())
}
