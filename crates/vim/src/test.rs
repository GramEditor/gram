mod vim_test_context;

use std::{sync::Arc, time::Duration};

use collections::HashMap;
use command_palette::CommandPalette;
use editor::{
    AnchorRangeExt, DisplayPoint, Editor, EditorMode, MultiBuffer, MultiBufferOffset, actions::WrapSelectionsInTag,
    code_context_menus::CodeContextMenu, display_map::DisplayRow, test::editor_test_context::EditorTestContext,
};
use futures::StreamExt;
use gpui::{KeyBinding, Modifiers, MouseButton, TestAppContext, UpdateGlobal as _, px};
use language::{CursorShape, Language, LanguageConfig, Point};
use settings::SettingsStore;
use ui::Pixels;
use util::test::marked_text_ranges;
pub use vim_test_context::*;

use indoc::indoc;
use search::BufferSearchBar;

use crate::{PushSneak, PushSneakBackward, insert::NormalBefore, state::Mode};

use util_macros::perf;

#[perf]
#[gpui::test]
async fn test_initially_disabled(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, false).await;
    cx.simulate_keystrokes("h j k l");
    cx.assert_editor_state("hjklˇ");
}

#[perf]
#[gpui::test]
async fn test_toggle_through_settings(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.simulate_keystrokes("i");
    assert_eq!(cx.mode(), Mode::Insert);

    // Editor acts as though vim is disabled
    cx.disable_vim();
    cx.simulate_keystrokes("h j k l");
    cx.assert_editor_state("hjklˇ");

    // Selections aren't changed if editor is blurred but vim-mode is still disabled.
    cx.cx.set_state("«hjklˇ»");
    cx.assert_editor_state("«hjklˇ»");
    cx.update_editor(|_, window, _cx| window.blur());
    cx.assert_editor_state("«hjklˇ»");
    cx.update_editor(|_, window, cx| cx.focus_self(window));
    cx.assert_editor_state("«hjklˇ»");

    // Enabling dynamically sets vim mode again and restores normal mode
    cx.enable_vim();
    assert_eq!(cx.mode(), Mode::Normal);
    cx.simulate_keystrokes("h h h l");
    assert_eq!(cx.buffer_text(), "hjkl".to_owned());
    cx.assert_editor_state("hˇjkl");
    cx.simulate_keystrokes("i T e s t");
    cx.assert_editor_state("hTestˇjkl");

    // Disabling and enabling resets to normal mode
    assert_eq!(cx.mode(), Mode::Insert);
    cx.disable_vim();
    cx.enable_vim();
    assert_eq!(cx.mode(), Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_cancel_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"The quick brown fox juˇmps over the lazy dog"}, Mode::Normal);
    // jumps
    cx.simulate_keystrokes("v l l");
    cx.assert_editor_state("The quick brown fox ju«mpsˇ» over the lazy dog");

    cx.simulate_keystrokes("escape");
    cx.assert_editor_state("The quick brown fox jumpˇs over the lazy dog");

    // go back to the same selection state
    cx.simulate_keystrokes("v h h");
    cx.assert_editor_state("The quick brown fox ju«ˇmps» over the lazy dog");

    // Ctrl-[ should behave like Esc
    cx.simulate_keystrokes("ctrl-[");
    cx.assert_editor_state("The quick brown fox juˇmps over the lazy dog");
}

#[perf]
#[gpui::test]
async fn test_buffer_search(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
            The quick brown
            fox juˇmps over
            the lazy dog"},
        Mode::Normal,
    );
    cx.simulate_keystrokes("/");

    let search_bar = cx.workspace(|workspace, _, cx| {
        workspace
            .active_pane()
            .read(cx)
            .toolbar()
            .read(cx)
            .item_of_type::<BufferSearchBar>()
            .expect("Buffer search bar should be deployed")
    });

    cx.update_entity(search_bar, |bar, _, cx| {
        assert_eq!(bar.query(cx), "");
    })
}

#[perf]
#[gpui::test]
async fn test_count_down(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"aˇa\nbb\ncc\ndd\nee"}, Mode::Normal);
    cx.simulate_keystrokes("2 down");
    cx.assert_editor_state("aa\nbb\ncˇc\ndd\nee");
    cx.simulate_keystrokes("9 down");
    cx.assert_editor_state("aa\nbb\ncc\ndd\neˇe");
}

#[perf]
#[gpui::test]
async fn test_end_of_document_710(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // goes to end by default
    cx.set_state(indoc! {"aˇa\nbb\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("shift-g");
    cx.assert_editor_state("aa\nbb\ncˇc");

    // can go to line 1 (https://github.com/zed-industries/zed/issues/5812)
    cx.simulate_keystrokes("1 shift-g");
    cx.assert_editor_state("aˇa\nbb\ncc");
}

#[perf]
#[gpui::test]
async fn test_end_of_line_with_times(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // goes to current line end
    cx.set_state(indoc! {"ˇaa\nbb\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("$");
    cx.assert_editor_state("aˇa\nbb\ncc");

    // goes to next line end
    cx.simulate_keystrokes("2 $");
    cx.assert_editor_state("aa\nbˇb\ncc");

    // try to exceed the final line.
    cx.simulate_keystrokes("4 $");
    cx.assert_editor_state("aa\nbb\ncˇc");
}

#[perf]
#[gpui::test]
async fn test_indent_outdent(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // works in normal mode
    cx.set_state(indoc! {"aa\nbˇb\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("> >");
    cx.assert_editor_state("aa\n    bˇb\ncc");
    cx.simulate_keystrokes("< <");
    cx.assert_editor_state("aa\nbˇb\ncc");

    // works in visual mode
    cx.simulate_keystrokes("shift-v down >");
    cx.assert_editor_state("aa\n    bˇb\n    cc");

    // works as operator
    cx.set_state("aa\nbˇb\ncc\n", Mode::Normal);
    cx.simulate_keystrokes("> j");
    cx.assert_editor_state("aa\n    bˇb\n    cc\n");
    cx.simulate_keystrokes("< k");
    cx.assert_editor_state("aa\nbˇb\n    cc\n");
    cx.simulate_keystrokes("> i p");
    cx.assert_editor_state("    aa\n    bˇb\n        cc\n");
    cx.simulate_keystrokes("< i p");
    cx.assert_editor_state("aa\nbˇb\n    cc\n");
    cx.simulate_keystrokes("< i p");
    cx.assert_editor_state("aa\nbˇb\ncc\n");

    cx.set_state("ˇaa\nbb\ncc\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 j");
    cx.assert_editor_state("    ˇaa\n    bb\n    cc\n");

    cx.set_state("aa\nbb\nˇcc\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 k");
    cx.assert_editor_state("    aa\n    bb\n    ˇcc\n");

    // works with repeat
    cx.set_state("a\nb\nccˇc\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 k");
    cx.assert_editor_state("    a\n    b\n    ccˇc\n");
    cx.simulate_keystrokes(".");
    cx.assert_editor_state("        a\n        b\n        ccˇc\n");
    cx.simulate_keystrokes("v k <");
    cx.assert_editor_state("        a\n    bˇ\n    ccc\n");
    cx.simulate_keystrokes(".");
    cx.assert_editor_state("        a\nbˇ\nccc\n");
}

#[perf]
#[gpui::test]
async fn test_escape_command_palette(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("aˇbc\n", Mode::Normal);
    cx.simulate_keystrokes("i cmd-shift-p");

    assert!(cx.workspace(|workspace, _, cx| workspace.active_modal::<CommandPalette>(cx).is_some()));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!cx.workspace(|workspace, _, cx| workspace.active_modal::<CommandPalette>(cx).is_some()));
    cx.assert_state("aˇbc\n", Mode::Insert);
}

#[perf]
#[gpui::test]
async fn test_escape_cancels(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("aˇbˇc", Mode::Normal);
    cx.simulate_keystrokes("escape");

    cx.assert_state("aˇbc", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_selection_on_search(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"aa\nbˇb\ncc\ncc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("/ c c");
    cx.wait_for_search();

    let search_bar = cx.workspace(|workspace, _, cx| {
        workspace
            .active_pane()
            .read(cx)
            .toolbar()
            .read(cx)
            .item_of_type::<BufferSearchBar>()
            .expect("Buffer search bar should be deployed")
    });

    cx.update_entity(search_bar, |bar, _, cx| {
        assert_eq!(bar.query(cx), "cc");
    });

    cx.update_editor(|editor, window, cx| {
        let highlights = editor.all_text_background_highlights(window, cx);
        assert_eq!(3, highlights.len());
        assert_eq!(
            DisplayPoint::new(DisplayRow(2), 0)..DisplayPoint::new(DisplayRow(2), 2),
            highlights[0].0
        )
    });
    cx.simulate_keystrokes("enter");

    cx.assert_state(indoc! {"aa\nbb\nˇcc\ncc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("n");
    cx.assert_state(indoc! {"aa\nbb\ncc\nˇcc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("shift-n");
    cx.assert_state(indoc! {"aa\nbb\nˇcc\ncc\ncc\n"}, Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_word_characters(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;
    cx.set_state(
        indoc! { "
        class A {
            #ˇgoop = 99;
            $ˇgoop () { return this.#gˇoop };
        };
        console.log(new A().$gooˇp())
    "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("v i w");
    cx.assert_state(
        indoc! {"
        class A {
            «#goopˇ» = 99;
            «$goopˇ» () { return this.«#goopˇ» };
        };
        console.log(new A().«$goopˇ»())
    "},
        Mode::Visual,
    )
}

#[perf]
#[gpui::test]
async fn test_kebab_case(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_html(cx).await;
    cx.set_state(
        indoc! { r#"
            <div><a class="bg-rˇed"></a></div>
            "#},
        Mode::Normal,
    );
    cx.simulate_keystrokes("v i w");
    cx.assert_state(
        indoc! { r#"
        <div><a class="bg-«redˇ»"></a></div>
        "#
        },
        Mode::Visual,
    )
}

#[perf]
#[gpui::test]
async fn test_select_all_issue_2170(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
        defmodule Test do
            def test(a, ˇ[_, _] = b), do: IO.puts('hi')
        end
    "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("g a");
    cx.assert_state(
        indoc! {"
        defmodule Test do
            def test(a, «[ˇ»_, _] = b), do: IO.puts('hi')
        end
    "},
        Mode::Visual,
    );
}

fn assert_pending_input(cx: &mut VimTestContext, expected: &str) {
    cx.update_editor(|editor, window, cx| {
        let snapshot = editor.snapshot(window, cx);
        let highlights = editor.text_highlights::<editor::PendingInput>(cx).unwrap().1;
        let (_, ranges) = marked_text_ranges(expected, false);

        assert_eq!(
            highlights
                .iter()
                .map(|highlight| highlight.to_offset(&snapshot.buffer_snapshot()))
                .collect::<Vec<_>>(),
            ranges
                .iter()
                .map(|range| MultiBufferOffset(range.start)..MultiBufferOffset(range.end))
                .collect::<Vec<_>>()
        )
    });
}

#[perf]
#[gpui::test]
async fn test_jk_multi(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update(|_, cx| cx.bind_keys([KeyBinding::new("j k l", NormalBefore, Some("vim_mode == insert"))]));

    cx.set_state("ˇone ˇone ˇone", Mode::Normal);
    cx.simulate_keystrokes("i j");
    cx.simulate_keystrokes("k");
    cx.assert_state("ˇjkone ˇjkone ˇjkone", Mode::Insert);
    assert_pending_input(&mut cx, "«jk»one «jk»one «jk»one");
    cx.simulate_keystrokes("o j k");
    cx.assert_state("jkoˇjkone jkoˇjkone jkoˇjkone", Mode::Insert);
    assert_pending_input(&mut cx, "jko«jk»one jko«jk»one jko«jk»one");
    cx.simulate_keystrokes("l");
    cx.assert_state("jkˇoone jkˇoone jkˇoone", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_jk_delay(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update(|_, cx| cx.bind_keys([KeyBinding::new("j k", NormalBefore, Some("vim_mode == insert"))]));

    cx.set_state("ˇhello", Mode::Normal);
    cx.simulate_keystrokes("i j");
    cx.executor().advance_clock(Duration::from_secs(16));
    cx.run_until_parked();
    cx.assert_state("ˇjhello", Mode::Insert);
    cx.update_editor(|editor, window, cx| {
        let snapshot = editor.snapshot(window, cx);
        let highlights = editor.text_highlights::<editor::PendingInput>(cx).unwrap().1;

        assert_eq!(
            highlights
                .iter()
                .map(|highlight| highlight.to_offset(&snapshot.buffer_snapshot()))
                .collect::<Vec<_>>(),
            vec![MultiBufferOffset(0)..MultiBufferOffset(1)]
        )
    });
    cx.executor().advance_clock(Duration::from_secs(16));
    cx.run_until_parked();
    cx.assert_state("jˇhello", Mode::Insert);
    cx.simulate_keystrokes("k j k");
    cx.assert_state("jˇkhello", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_completion_menu_scroll_aside(cx: &mut TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;
    cx.update(|_, cx| {
        SettingsStore::update_global(cx, |store, cx| {
            store.update_user_settings(cx, |settings| {
                settings.project.all_languages.defaults.show_completions_on_input = Some(true);
            });
        });
    });

    cx.lsp
        .set_request_handler::<lsp::request::Completion, _, _>(move |_, _| async move {
            Ok(Some(lsp::CompletionResponse::Array(vec![
                lsp::CompletionItem {
                    label: "Test Item".to_string(),
                    documentation: Some(lsp::Documentation::String(
                        "This is some very long documentation content that will be displayed in the aside panel for scrolling.\n".repeat(50)
                    )),
                    ..Default::default()
                },
            ])))
        });

    cx.set_state("variableˇ", Mode::Insert);
    cx.simulate_keystroke(".");
    cx.executor().run_until_parked();

    let mut initial_offset: Pixels = px(0.0);

    cx.update_editor(|editor, _, _| {
        let binding = editor.context_menu().borrow();
        let Some(CodeContextMenu::Completions(menu)) = binding.as_ref() else {
            panic!("Should have completions menu open");
        };

        initial_offset = menu.scroll_handle_aside.offset().y;
    });

    // The `ctrl-e` shortcut should scroll the completion menu's aside content
    // down, so the updated offset should be lower than the initial offset.
    cx.simulate_keystroke("ctrl-e");
    cx.update_editor(|editor, _, _| {
        let binding = editor.context_menu().borrow();
        let Some(CodeContextMenu::Completions(menu)) = binding.as_ref() else {
            panic!("Should have completions menu open");
        };

        assert!(menu.scroll_handle_aside.offset().y < initial_offset);
    });

    // The `ctrl-y` shortcut should do the inverse scrolling as `ctrl-e`, so the
    // offset should now be the same as the initial offset.
    cx.simulate_keystroke("ctrl-y");
    cx.update_editor(|editor, _, _| {
        let binding = editor.context_menu().borrow();
        let Some(CodeContextMenu::Completions(menu)) = binding.as_ref() else {
            panic!("Should have completions menu open");
        };

        assert_eq!(menu.scroll_handle_aside.offset().y, initial_offset);
    });

    // The `ctrl-d` shortcut should scroll the completion menu's aside content
    // down, so the updated offset should be lower than the initial offset.
    cx.simulate_keystroke("ctrl-d");
    cx.update_editor(|editor, _, _| {
        let binding = editor.context_menu().borrow();
        let Some(CodeContextMenu::Completions(menu)) = binding.as_ref() else {
            panic!("Should have completions menu open");
        };

        assert!(menu.scroll_handle_aside.offset().y < initial_offset);
    });

    // The `ctrl-u` shortcut should do the inverse scrolling as `ctrl-u`, so the
    // offset should now be the same as the initial offset.
    cx.simulate_keystroke("ctrl-u");
    cx.update_editor(|editor, _, _| {
        let binding = editor.context_menu().borrow();
        let Some(CodeContextMenu::Completions(menu)) = binding.as_ref() else {
            panic!("Should have completions menu open");
        };

        assert_eq!(menu.scroll_handle_aside.offset().y, initial_offset);
    });
}

#[perf]
#[gpui::test]
async fn test_rename(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;

    cx.set_state("const beˇfore = 2; console.log(before)", Mode::Normal);
    let def_range = cx.lsp_range("const «beforeˇ» = 2; console.log(before)");
    let tgt_range = cx.lsp_range("const before = 2; console.log(«beforeˇ»)");
    let mut prepare_request =
        cx.set_request_handler::<lsp::request::PrepareRenameRequest, _, _>(move |_, _, _| async move {
            Ok(Some(lsp::PrepareRenameResponse::Range(def_range)))
        });
    let mut rename_request = cx.set_request_handler::<lsp::request::Rename, _, _>(move |url, params, _| async move {
        Ok(Some(lsp::WorkspaceEdit {
            changes: Some(
                [(
                    url.clone(),
                    vec![
                        lsp::TextEdit::new(def_range, params.new_name.clone()),
                        lsp::TextEdit::new(tgt_range, params.new_name),
                    ],
                )]
                .into(),
            ),
            ..Default::default()
        }))
    });

    cx.simulate_keystrokes("c d");
    prepare_request.next().await.unwrap();
    cx.simulate_input("after");
    cx.simulate_keystrokes("enter");
    rename_request.next().await.unwrap();
    cx.assert_state("const afterˇ = 2; console.log(after)", Mode::Normal)
}

#[gpui::test]
async fn test_go_to_definition(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;

    cx.set_state("const before = 2; console.log(beforˇe)", Mode::Normal);
    let def_range = cx.lsp_range("const «beforeˇ» = 2; console.log(before)");
    let mut go_to_request = cx.set_request_handler::<lsp::request::GotoDefinition, _, _>(move |url, _, _| async move {
        Ok(Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location::new(
            url.clone(),
            def_range,
        ))))
    });

    cx.simulate_keystrokes("g d");
    go_to_request.next().await.unwrap();
    cx.run_until_parked();

    cx.assert_state("const ˇbefore = 2; console.log(before)", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_remap(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // test moving the cursor
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g z",
            workspace::SendKeystrokes("l l l l".to_string()),
            None,
        )])
    });
    cx.set_state("ˇ123456789", Mode::Normal);
    cx.simulate_keystrokes("g z");
    cx.assert_state("1234ˇ56789", Mode::Normal);

    // test switching modes
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g y",
            workspace::SendKeystrokes("i f o o escape l".to_string()),
            None,
        )])
    });
    cx.set_state("ˇ123456789", Mode::Normal);
    cx.simulate_keystrokes("g y");
    cx.assert_state("fooˇ123456789", Mode::Normal);

    // test recursion
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g x",
            workspace::SendKeystrokes("g z g y".to_string()),
            None,
        )])
    });
    cx.set_state("ˇ123456789", Mode::Normal);
    cx.simulate_keystrokes("g x");
    cx.assert_state("1234fooˇ56789", Mode::Normal);

    // test command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g w",
            workspace::SendKeystrokes(": j enter".to_string()),
            None,
        )])
    });
    cx.set_state("ˇ1234\n56789", Mode::Normal);
    cx.simulate_keystrokes("g w");
    cx.assert_state("1234ˇ 56789", Mode::Normal);

    // test leaving command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g u",
            workspace::SendKeystrokes("g w g z".to_string()),
            None,
        )])
    });
    cx.set_state("ˇ1234\n56789", Mode::Normal);
    cx.simulate_keystrokes("g u");
    cx.assert_state("1234 567ˇ89", Mode::Normal);

    // test leaving command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g t",
            workspace::SendKeystrokes("i space escape".to_string()),
            None,
        )])
    });
    cx.set_state("12ˇ34", Mode::Normal);
    cx.simulate_keystrokes("g t");
    cx.assert_state("12ˇ 34", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_mouse_selection(cx: &mut TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("ˇone two three", Mode::Normal);

    let start_point = cx.pixel_position("one twˇo three");
    let end_point = cx.pixel_position("one ˇtwo three");

    cx.simulate_mouse_down(start_point, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end_point, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end_point, MouseButton::Left, Modifiers::none());

    cx.assert_state("one «ˇtwo» three", Mode::Visual)
}

#[gpui::test]
async fn test_mouse_drag_across_anchor_does_not_drift(cx: &mut TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("ˇone two three four", Mode::Normal);

    let click_pos = cx.pixel_position("one ˇtwo three four");
    let drag_left = cx.pixel_position("ˇone two three four");
    let anchor_pos = cx.pixel_position("one tˇwo three four");

    cx.simulate_mouse_down(click_pos, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    cx.simulate_mouse_move(drag_left, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.assert_state("«ˇone t»wo three four", Mode::Visual);

    cx.simulate_mouse_move(anchor_pos, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    cx.simulate_mouse_move(drag_left, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.assert_state("«ˇone t»wo three four", Mode::Visual);

    cx.simulate_mouse_move(anchor_pos, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_move(drag_left, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    cx.assert_state("«ˇone t»wo three four", Mode::Visual);

    cx.simulate_mouse_up(drag_left, MouseButton::Left, Modifiers::none());
}

#[perf]
#[gpui::test]
async fn test_toggle_comments(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    let language = std::sync::Arc::new(language::Language::new(
        language::LanguageConfig {
            line_comments: vec!["// ".into(), "//! ".into(), "/// ".into()],
            ..Default::default()
        },
        Some(language::tree_sitter_rust::LANGUAGE.into()),
    ));
    cx.update_buffer(|buffer, cx| buffer.set_language(Some(language), cx));

    // works in normal model
    cx.set_state(
        indoc! {"
      ˇone
      two
      three
      "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("g c c");
    cx.assert_state(
        indoc! {"
          // ˇone
          two
          three
          "},
        Mode::Normal,
    );

    // works in visual mode
    cx.simulate_keystrokes("v j g c");
    cx.assert_state(
        indoc! {"
          // // ˇone
          // two
          three
          "},
        Mode::Normal,
    );

    // works in visual line mode
    cx.simulate_keystrokes("shift-v j g c");
    cx.assert_state(
        indoc! {"
          // ˇone
          two
          three
          "},
        Mode::Normal,
    );

    // works with count
    cx.simulate_keystrokes("g c 2 j");
    cx.assert_state(
        indoc! {"
            // // ˇone
            // two
            // three
            "},
        Mode::Normal,
    );

    // works with motion object
    cx.simulate_keystrokes("shift-g");
    cx.simulate_keystrokes("g c g g");
    cx.assert_state(
        indoc! {"
            // one
            two
            three
            ˇ"},
        Mode::Normal,
    );
}

#[perf]
#[gpui::test]
async fn test_sneak(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update(|_window, cx| {
        cx.bind_keys([
            KeyBinding::new("s", PushSneak { first_char: None }, Some("vim_mode == normal")),
            KeyBinding::new(
                "shift-s",
                PushSneakBackward { first_char: None },
                Some("vim_mode == normal"),
            ),
            KeyBinding::new(
                "shift-s",
                PushSneakBackward { first_char: None },
                Some("vim_mode == visual"),
            ),
        ])
    });

    // Sneak forwards multibyte & multiline
    cx.set_state(
        indoc! {
            r#"<labelˇ for="guests">
                    Počet hostů
                </label>"#
        },
        Mode::Normal,
    );
    cx.simulate_keystrokes("s t ů");
    cx.assert_state(
        indoc! {
            r#"<label for="guests">
                Počet hosˇtů
            </label>"#
        },
        Mode::Normal,
    );

    // Visual sneak backwards multibyte & multiline
    cx.simulate_keystrokes("v S < l");
    cx.assert_state(
        indoc! {
            r#"«ˇ<label for="guests">
                Počet host»ů
            </label>"#
        },
        Mode::Visual,
    );

    // Sneak backwards repeated
    cx.set_state(r#"11 12 13 ˇ14"#, Mode::Normal);
    cx.simulate_keystrokes("S space 1");
    cx.assert_state(r#"11 12ˇ 13 14"#, Mode::Normal);
    cx.simulate_keystrokes(";");
    cx.assert_state(r#"11ˇ 12 13 14"#, Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_command_alias(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;
    cx.update_global(|store: &mut SettingsStore, cx| {
        store.update_user_settings(cx, |s| {
            let mut aliases = HashMap::default();
            aliases.insert("Q".to_string(), "upper".to_string());
            s.workspace.command_aliases = aliases
        });
    });

    cx.set_state("ˇhello world", Mode::Normal);
    cx.simulate_keystrokes(": Q");
    cx.set_state("ˇHello world", Mode::Normal);
}

#[perf]
#[gpui::test]
async fn test_visual_indent_count(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;
    cx.set_state("ˇhi", Mode::Normal);
    cx.simulate_keystrokes("shift-v 3 >");
    cx.assert_state("            ˇhi", Mode::Normal);
    cx.simulate_keystrokes("shift-v 2 <");
    cx.assert_state("    ˇhi", Mode::Normal);
}

#[perf(iterations = 1)]
#[gpui::test]
async fn test_folded_multibuffer_excerpts(cx: &mut gpui::TestAppContext) {
    VimTestContext::init(cx);
    cx.update(|cx| {
        VimTestContext::init_keybindings(true, cx);
    });
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let multi_buffer = MultiBuffer::build_multi(
            [
                ("111\n222\n333\n444\n", vec![Point::row_range(0..2)]),
                ("aaa\nbbb\nccc\nddd\n", vec![Point::row_range(0..2)]),
                ("AAA\nBBB\nCCC\nDDD\n", vec![Point::row_range(0..2)]),
                ("one\ntwo\nthr\nfou\n", vec![Point::row_range(0..2)]),
            ],
            cx,
        );
        let mut editor = Editor::new(EditorMode::full(), multi_buffer.clone(), None, window, cx);

        let buffer_ids = multi_buffer.read(cx).excerpt_buffer_ids();
        // fold all but the second buffer, so that we test navigating between two
        // adjacent folded buffers, as well as folded buffers at the start and
        // end the multibuffer
        editor.fold_buffer(buffer_ids[0], cx);
        editor.fold_buffer(buffer_ids[2], cx);
        editor.fold_buffer(buffer_ids[3], cx);

        editor
    });
    let mut cx = EditorTestContext::for_editor_in(editor.clone(), cx).await;

    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        ˇaaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        ˇ[EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        ˇ[FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.simulate_keystroke("k");
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        ˇaaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("shift-g");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        ˇ[FOLDED]
        "
    });
    cx.simulate_keystrokes("g g");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.update_editor(|editor, _, cx| {
        let buffer_ids = editor.buffer().read(cx).excerpt_buffer_ids();
        editor.fold_buffer(buffer_ids[1], cx);
    });

    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystrokes("2 j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        ˇ[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
}

#[perf]
#[gpui::test]
async fn test_multi_cursor_replay(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;
    cx.set_state(
        indoc! {
            "
        oˇne one one

        two two two
        "
        },
        Mode::Normal,
    );

    cx.simulate_keystrokes("3 g l s wow escape escape");
    cx.assert_state(
        indoc! {
            "
        woˇw wow wow

        two two two
        "
        },
        Mode::Normal,
    );

    cx.simulate_keystrokes("2 j 3 g l .");
    cx.assert_state(
        indoc! {
            "
        wow wow wow

        woˇw woˇw woˇw
        "
        },
        Mode::Normal,
    );
}

#[gpui::test]
async fn test_clipping_on_mode_change(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {
        "
        ˇverylongline
        andsomelinebelow
        "
        },
        Mode::Normal,
    );

    cx.simulate_keystrokes("v e");
    cx.assert_state(
        indoc! {
        "
        «verylonglineˇ»
        andsomelinebelow
        "
        },
        Mode::Visual,
    );

    let mut pixel_position = cx.update_editor(|editor, window, cx| {
        let snapshot = editor.snapshot(window, cx);
        let current_head = editor.selections.newest_display(&snapshot.display_snapshot).end;
        editor.last_bounds().unwrap().origin
            + editor
                .display_to_pixel_point(current_head, &snapshot, window, cx)
                .unwrap()
    });
    pixel_position.x += px(100.);
    // click beyond end of the line
    cx.simulate_click(pixel_position, Modifiers::default());
    cx.run_until_parked();

    cx.assert_state(
        indoc! {
        "
        verylonglinˇe
        andsomelinebelow
        "
        },
        Mode::Normal,
    );
}

#[gpui::test]
async fn test_wrap_selections_in_tag_line_mode(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    let js_language = Arc::new(Language::new(
        LanguageConfig {
            name: "JavaScript".into(),
            wrap_characters: Some(language::WrapCharactersConfig {
                start_prefix: "<".into(),
                start_suffix: ">".into(),
                end_prefix: "</".into(),
                end_suffix: ">".into(),
            }),
            ..LanguageConfig::default()
        },
        None,
    ));

    cx.update_buffer(|buffer, cx| buffer.set_language(Some(js_language), cx));

    cx.set_state(
        indoc! {
        "
        ˇaaaaa
        bbbbb
        "
        },
        Mode::Normal,
    );

    cx.simulate_keystrokes("shift-v j");
    cx.dispatch_action(WrapSelectionsInTag);

    cx.assert_state(
        indoc! {
            "
            <ˇ>aaaaa
            bbbbb</ˇ>
            "
        },
        Mode::VisualLine,
    );
}

#[gpui::test]
async fn test_deactivate(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update_global(|store: &mut SettingsStore, cx| {
        store.update_user_settings(cx, |settings| {
            settings.editor.cursor_shape = Some(settings::CursorShape::Underline);
        });
    });

    // Assert that, while in `Normal` mode, the cursor shape is `Block` but,
    // after deactivating vim mode, it should revert to the one specified in the
    // user's settings, if set.
    cx.update_editor(|editor, _window, _cx| {
        assert_eq!(editor.cursor_shape(), CursorShape::Block);
    });

    cx.disable_vim();

    cx.update_editor(|editor, _window, _cx| {
        assert_eq!(editor.cursor_shape(), CursorShape::Underline);
    });
}
