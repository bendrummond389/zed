## Notes

The `conversationeditor` has a `completionprovider`.

`new_inline_assistant` implements the following:
`Note the new view`

```rust
let inline_assistant = cx.new_view(|cx| {
    InlineAssistant::new(
        inline_assist_id,
        measurements.clone(),
        self.include_conversation_in_next_inline_assist,
        self.inline_prompt_history.clone(),
        codegen.clone(),
        self.workspace.clone(),
        cx,
        self.retrieve_context_in_next_inline_assist,
        self.semantic_index.clone(),
        project.clone(),
    )
});
```

Also, new_inline_assist uses codegen which utilizes:

```rust
let provider = self.completion_provider.clone();
```

Function `new_inline_assist` details:

```rust
fn new_inline_assist(
    &mut self,
    editor: &View<Editor>,
    cx: &mut ViewContext<Self>,
    project: &Model<Project>,
) {
    let selection = editor.read(cx).selections.newest_anchor().clone();
    if selection.start.excerpt_id != selection.end.excerpt_id {
        return;
    }
    let snapshot = editor.read(cx).buffer().read(cx).snapshot(cx);

    // Extend the selection to the start and the end of the line.
    let mut point_selection = selection.map(|selection| selection.to_point(&snapshot));
    if point_selection.end > point_selection.start {
        point_selection.start.column = 0;
        // If the selection ends at the start of the line, we don't want to include it.
        if point_selection.end.column == 0 {
            point_selection.end.row -= 1;
        }
        point_selection.end.column = snapshot.line_len(point_selection.end.row);
    }

    let codegen_kind = if point_selection.start == point_selection.end {
        CodegenKind::Generate {
            position: snapshot.anchor_after(point_selection.start),
        }
    } else {
        CodegenKind::Transform {
            range: snapshot.anchor_before(point_selection.start)
                ..snapshot.anchor_after(point_selection.end),
        }
    };

    let inline_assist_id = post_inc(&mut self.next_inline_assist_id);
    let provider = self.completion_provider.clone();

    let codegen = cx.new_model(|cx| {
        Codegen::new(editor.read(cx).buffer().clone(), codegen_kind, provider, cx)
    });
```

## The assistant button is implemented as:

```rust
let assistant_button = QuickActionBarButton::new(
      "toggle inline assistant",
      IconName::MagicWand,
      false,
      Box::new(InlineAssist),
      "Inline Assist",
      {
          let workspace = self.workspace.clone();
          move |_, cx| {
              if let Some(workspace) = workspace.upgrade() {
                  workspace.update(cx, |workspace, cx| {
                      AssistantPanel::inline_assist(workspace, &InlineAssist, cx);
                  });
              }
          }
      },
  );
```

```rust

pub fn inline_assist(
    workspace: &mut Workspace,
    _: &InlineAssist,
    cx: &mut ViewContext<Workspace>,
) {
    let Some(assistant) = workspace.panel::<AssistantPanel>(cx) else {
        return;
    };
    let active_editor = if let Some(active_editor) = workspace
        .active_item(cx)
        .and_then(|item| item.act_as::<Editor>(cx))
    {
        active_editor
    } else {
        return;
    };
    let project = workspace.project().clone();

    if assistant.update(cx, |assistant, _| assistant.has_credentials()) {
        assistant.update(cx, |assistant, cx| {
            assistant.new_inline_assist(&active_editor, cx, &project)
        });
    } else {
        let assistant = assistant.downgrade();
        cx.spawn(|workspace, mut cx| async move {
            assistant
                .update(&mut cx, |assistant, cx| assistant.load_credentials(cx))?
                .await;
            if assistant.update(&mut cx, |assistant, _| assistant.has_credentials())? {
                assistant.update(&mut cx, |assistant, cx| {
                    assistant.new_inline_assist(&active_editor, cx, &project)
                })?;
            } else {
                workspace.update(&mut cx, |workspace, cx| {
                    workspace.focus_panel::<AssistantPanel>(cx)
                })?;
            }

            anyhow::Ok(())
        })
        .detach_and_log_err(cx)
    }
}

```

```rust

cx.spawn(|buffer, mut cx| async move {
    let markdown = markdown.await?;
    buffer.update(&mut cx, |buffer: &mut Buffer, cx| {
        buffer.set_language(Some(markdown), cx)
    })?;
    anyhow::Ok(())
})
```
