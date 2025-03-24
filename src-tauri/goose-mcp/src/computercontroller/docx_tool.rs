use docx_rs::*;
use image::{self, ImageFormat};
use mcp_core::{Content, ToolError};
use std::{fs, io::Cursor};

#[derive(Debug)]
enum UpdateMode {
    Append,
    Replace {
        old_text: String,
    },
    InsertStructured {
        level: Option<String>, // e.g., "Heading1", "Heading2", etc.
        style: Option<DocxStyle>,
    },
    AddImage {
        image_path: String,
        width: Option<u32>,
        height: Option<u32>,
    },
}

#[derive(Debug, Clone, Default)]
struct DocxStyle {
    bold: bool,
    italic: bool,
    underline: bool,
    size: Option<usize>,
    color: Option<String>,
    alignment: Option<AlignmentType>,
}

impl DocxStyle {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        Some(Self {
            bold: obj.get("bold").and_then(|v| v.as_bool()).unwrap_or(false),
            italic: obj.get("italic").and_then(|v| v.as_bool()).unwrap_or(false),
            underline: obj
                .get("underline")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            size: obj.get("size").and_then(|v| v.as_u64()).map(|s| s as usize),
            color: obj.get("color").and_then(|v| v.as_str()).map(String::from),
            alignment: obj
                .get("alignment")
                .and_then(|v| v.as_str())
                .and_then(|a| match a {
                    "left" => Some(AlignmentType::Left),
                    "center" => Some(AlignmentType::Center),
                    "right" => Some(AlignmentType::Right),
                    "justified" => Some(AlignmentType::Both),
                    _ => None,
                }),
        })
    }

    fn apply_to_run(&self, run: Run) -> Run {
        let mut run = run;
        if self.bold {
            run = run.bold();
        }
        if self.italic {
            run = run.italic();
        }
        if self.underline {
            run = run.underline("single");
        }
        if let Some(size) = self.size {
            run = run.size(size);
        }
        if let Some(color) = &self.color {
            run = run.color(color);
        }
        run
    }

    fn apply_to_paragraph(&self, para: Paragraph) -> Paragraph {
        let mut para = para;
        if let Some(alignment) = self.alignment {
            para = para.align(alignment);
        }
        para
    }
}

pub async fn docx_tool(
    path: &str,
    operation: &str,
    content: Option<&str>,
    params: Option<&serde_json::Value>,
) -> Result<Vec<Content>, ToolError> {
    match operation {
        "extract_text" => {
            let file = fs::read(path).map_err(|e| {
                ToolError::ExecutionError(format!("Failed to read DOCX file: {}", e))
            })?;

            let docx = read_docx(&file).map_err(|e| {
                ToolError::ExecutionError(format!("Failed to parse DOCX file: {}", e))
            })?;

            let mut text = String::new();
            let mut structure = Vec::new();
            let mut current_level = None;

            // Extract document structure and text
            for element in docx.document.children.iter() {
                if let DocumentChild::Paragraph(p) = element {
                    // Check for heading style
                    if let Some(style) = p.property.style.as_ref() {
                        if style.val.starts_with("Heading") {
                            current_level = Some(style.val.clone());
                            structure.push(format!("{}: ", style.val));
                        }
                    }

                    // Extract text from runs
                    let para_text: String = p
                        .children
                        .iter()
                        .filter_map(|child| {
                            if let ParagraphChild::Run(run) = child {
                                Some(
                                    run.children
                                        .iter()
                                        .filter_map(|rc| {
                                            if let RunChild::Text(t) = rc {
                                                Some(t.text.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join(""),
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    if !para_text.trim().is_empty() {
                        if current_level.is_some() {
                            if let Some(s) = structure.last_mut() {
                                s.push_str(&para_text);
                            }
                            current_level = None;
                        }
                        text.push_str(&para_text);
                        text.push('\n');
                    }
                }
            }

            let result = if !structure.is_empty() {
                format!(
                    "Document Structure:\n{}\n\nFull Text:\n{}",
                    structure.join("\n"),
                    text
                )
            } else {
                format!("Extracted Text:\n{}", text)
            };

            Ok(vec![Content::text(result)])
        }

        "update_doc" => {
            let content = content.ok_or_else(|| {
                ToolError::InvalidParameters(
                    "Content parameter required for update_doc".to_string(),
                )
            })?;

            // Parse update mode and style from params
            let (mode, style) = if let Some(params) = params {
                let mode = params
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("append");
                let style = params.get("style").and_then(DocxStyle::from_json);

                let mode = match mode {
                    "append" => UpdateMode::Append,
                    "replace" => {
                        let old_text =
                            params
                                .get("old_text")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| {
                                    ToolError::InvalidParameters(
                                        "old_text parameter required for replace mode".to_string(),
                                    )
                                })?;
                        UpdateMode::Replace {
                            old_text: old_text.to_string(),
                        }
                    }
                    "structured" => {
                        let level = params
                            .get("level")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        UpdateMode::InsertStructured {
                            level,
                            style: style.clone(),
                        }
                    }
                    "add_image" => {
                        let image_path = params
                            .get("image_path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                ToolError::InvalidParameters(
                                    "image_path parameter required for add_image mode".to_string(),
                                )
                            })?
                            .to_string();

                        let width = params
                            .get("width")
                            .and_then(|v| v.as_u64())
                            .map(|w| w as u32);

                        let height = params
                            .get("height")
                            .and_then(|v| v.as_u64())
                            .map(|h| h as u32);

                        UpdateMode::AddImage {
                            image_path,
                            width,
                            height,
                        }
                    }
                    _ => return Err(ToolError::InvalidParameters(
                        "Invalid mode. Must be 'append', 'replace', 'structured', or 'add_image'"
                            .to_string(),
                    )),
                };
                (mode, style)
            } else {
                (UpdateMode::Append, None)
            };

            match mode {
                UpdateMode::Append => {
                    // Read existing document if it exists, or create new one
                    let mut doc = if std::path::Path::new(path).exists() {
                        let file = fs::read(path).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to read DOCX file: {}", e))
                        })?;
                        read_docx(&file).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to parse DOCX file: {}", e))
                        })?
                    } else {
                        Docx::new()
                    };

                    // Split content into paragraphs and add them
                    for para in content.split('\n') {
                        if !para.trim().is_empty() {
                            let mut run = Run::new().add_text(para);
                            let mut paragraph = Paragraph::new();

                            if let Some(style) = &style {
                                run = style.apply_to_run(run);
                                paragraph = style.apply_to_paragraph(paragraph);
                            }

                            doc = doc.add_paragraph(paragraph.add_run(run));
                        }
                    }

                    let mut buf = Vec::new();
                    {
                        let mut cursor = Cursor::new(&mut buf);
                        doc.build().pack(&mut cursor).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to build DOCX: {}", e))
                        })?;
                    }

                    fs::write(path, &buf).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to write DOCX file: {}", e))
                    })?;

                    Ok(vec![Content::text(format!(
                        "Successfully wrote content to {}",
                        path
                    ))])
                }

                UpdateMode::Replace { old_text } => {
                    // Read existing document
                    let file = fs::read(path).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to read DOCX file: {}", e))
                    })?;

                    let docx = read_docx(&file).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to parse DOCX file: {}", e))
                    })?;

                    let mut new_doc = Docx::new();
                    let mut found_text = false;

                    // Process each paragraph
                    for element in docx.document.children.iter() {
                        if let DocumentChild::Paragraph(p) = element {
                            let para_text: String = p
                                .children
                                .iter()
                                .filter_map(|child| {
                                    if let ParagraphChild::Run(run) = child {
                                        Some(
                                            run.children
                                                .iter()
                                                .filter_map(|rc| {
                                                    if let RunChild::Text(t) = rc {
                                                        Some(t.text.clone())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                                .join(""),
                                        )
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("");

                            if para_text.contains(&old_text) {
                                // Replace this paragraph with new content
                                found_text = true;
                                for para in content.split('\n') {
                                    if !para.trim().is_empty() {
                                        let mut run = Run::new().add_text(para);
                                        let mut paragraph = Paragraph::new();

                                        if let Some(style) = &style {
                                            run = style.apply_to_run(run);
                                            paragraph = style.apply_to_paragraph(paragraph);
                                        }

                                        new_doc = new_doc.add_paragraph(paragraph.add_run(run));
                                    }
                                }
                            } else {
                                // Create a new paragraph with the same content and style
                                let mut para = Paragraph::new();
                                if let Some(style) = &p.property.style {
                                    para = para.style(&style.val);
                                }
                                for child in p.children.iter() {
                                    if let ParagraphChild::Run(run) = child {
                                        for rc in run.children.iter() {
                                            if let RunChild::Text(t) = rc {
                                                para = para.add_run(Run::new().add_text(&t.text));
                                            }
                                        }
                                    }
                                }
                                new_doc = new_doc.add_paragraph(para);
                            }
                        }
                    }

                    if !found_text {
                        return Err(ToolError::ExecutionError(format!(
                            "Could not find text to replace: {}",
                            old_text
                        )));
                    }

                    let mut buf = Vec::new();
                    {
                        let mut cursor = Cursor::new(&mut buf);
                        new_doc.build().pack(&mut cursor).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to build DOCX: {}", e))
                        })?;
                    }

                    fs::write(path, &buf).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to write DOCX file: {}", e))
                    })?;

                    Ok(vec![Content::text(format!(
                        "Successfully replaced content in {}",
                        path
                    ))])
                }

                UpdateMode::InsertStructured { level, style } => {
                    let mut doc = if std::path::Path::new(path).exists() {
                        let file = fs::read(path).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to read DOCX file: {}", e))
                        })?;
                        read_docx(&file).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to parse DOCX file: {}", e))
                        })?
                    } else {
                        Docx::new()
                    };

                    // Create the paragraph with heading style if specified
                    for para in content.split('\n') {
                        if !para.trim().is_empty() {
                            let mut run = Run::new().add_text(para);
                            let mut paragraph = Paragraph::new();

                            // Apply heading style if specified
                            if let Some(level) = &level {
                                paragraph = paragraph.style(level);
                            }

                            // Apply custom style if specified
                            if let Some(style) = &style {
                                run = style.apply_to_run(run);
                                paragraph = style.apply_to_paragraph(paragraph);
                            }

                            doc = doc.add_paragraph(paragraph.add_run(run));
                        }
                    }

                    let mut buf = Vec::new();
                    {
                        let mut cursor = Cursor::new(&mut buf);
                        doc.build().pack(&mut cursor).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to build DOCX: {}", e))
                        })?;
                    }

                    fs::write(path, &buf).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to write DOCX file: {}", e))
                    })?;

                    Ok(vec![Content::text(format!(
                        "Successfully added structured content to {}",
                        path
                    ))])
                }

                UpdateMode::AddImage {
                    image_path,
                    width,
                    height,
                } => {
                    let mut doc = if std::path::Path::new(path).exists() {
                        let file = fs::read(path).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to read DOCX file: {}", e))
                        })?;
                        read_docx(&file).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to parse DOCX file: {}", e))
                        })?
                    } else {
                        Docx::new()
                    };

                    // Read the image file
                    let image_data = fs::read(&image_path).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to read image file: {}", e))
                    })?;

                    // Get image format and extension
                    let extension = std::path::Path::new(&image_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .ok_or_else(|| {
                            ToolError::ExecutionError("Invalid image file extension".to_string())
                        })?
                        .to_lowercase();

                    // Convert to PNG if not already PNG
                    let image_data = if extension != "png" {
                        // Try to convert to PNG using the image crate
                        let img = image::load_from_memory(&image_data).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to load image: {}", e))
                        })?;
                        let mut png_data = Vec::new();
                        img.write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png)
                            .map_err(|e| {
                                ToolError::ExecutionError(format!(
                                    "Failed to convert image to PNG: {}",
                                    e
                                ))
                            })?;
                        png_data
                    } else {
                        image_data
                    };

                    // Add optional caption if provided
                    if !content.trim().is_empty() {
                        let mut caption = Paragraph::new();
                        if let Some(style) = &style {
                            caption = style.apply_to_paragraph(caption);
                            caption =
                                caption.add_run(style.apply_to_run(Run::new().add_text(content)));
                        } else {
                            caption = caption.add_run(Run::new().add_text(content));
                        }
                        doc = doc.add_paragraph(caption);
                    }

                    // Create a paragraph with the image
                    let mut paragraph = Paragraph::new();
                    if let Some(style) = &style {
                        paragraph = style.apply_to_paragraph(paragraph);
                    }

                    // Create and add the image
                    let mut pic = Pic::new(&image_data);
                    if let (Some(w), Some(h)) = (width, height) {
                        pic = pic.size(w, h);
                    }

                    paragraph = paragraph.add_run(Run::new().add_image(pic));
                    doc = doc.add_paragraph(paragraph);

                    let mut buf = Vec::new();
                    {
                        let mut cursor = Cursor::new(&mut buf);
                        doc.build().pack(&mut cursor).map_err(|e| {
                            ToolError::ExecutionError(format!("Failed to build DOCX: {}", e))
                        })?;
                    }

                    fs::write(path, &buf).map_err(|e| {
                        ToolError::ExecutionError(format!("Failed to write DOCX file: {}", e))
                    })?;

                    Ok(vec![Content::text(format!(
                        "Successfully added image to {}",
                        path
                    ))])
                }
            }
        }

        _ => Err(ToolError::InvalidParameters(format!(
            "Invalid operation: {}. Valid operations are: 'extract_text', 'update_doc'",
            operation
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_docx_text_extraction() {
        let test_docx_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/sample.docx");

        println!("Testing text extraction from: {}", test_docx_path.display());

        let result = docx_tool(test_docx_path.to_str().unwrap(), "extract_text", None, None).await;

        assert!(result.is_ok(), "DOCX text extraction should succeed");
        let content = result.unwrap();
        assert!(!content.is_empty(), "Extracted text should not be empty");
        let text = content[0].as_text().unwrap();
        println!("Extracted text:\n{}", text);
        assert!(
            !text.trim().is_empty(),
            "Extracted text should not be empty"
        );
    }

    #[tokio::test]
    async fn test_docx_update_append() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_output.docx");

        let test_content =
            "Test Heading\nThis is a test paragraph.\n\nAnother paragraph with some content.";

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(test_content),
            None,
        )
        .await;

        assert!(result.is_ok(), "DOCX update should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Now try to read it back
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "Should be able to read back the written file"
        );
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();
        assert!(
            text.contains("Test Heading"),
            "Should contain written content"
        );
        assert!(
            text.contains("test paragraph"),
            "Should contain written content"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_update_styled() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_styled.docx");

        let test_content = "Styled Heading\nThis is a styled paragraph.";
        let params = json!({
            "mode": "structured",
            "level": "Heading1",
            "style": {
                "bold": true,
                "color": "FF0000",
                "size": 24,
                "alignment": "center"
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(test_content),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX styled update should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_update_replace() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_replace.docx");

        // First create a document
        let initial_content = "Original content\nThis should be replaced.\nKeep this text.";
        let _ = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(initial_content),
            None,
        )
        .await;

        // Now replace part of it
        let replacement = "New content here";
        let params = json!({
            "mode": "replace",
            "old_text": "This should be replaced",
            "style": {
                "italic": true
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(replacement),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX replace should succeed");

        // Verify the content
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();
        assert!(
            text.contains("New content here"),
            "Should contain new content"
        );
        assert!(
            text.contains("Keep this text"),
            "Should keep unmodified content"
        );
        assert!(
            !text.contains("This should be replaced"),
            "Should not contain replaced text"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_add_image() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_image.docx");

        // Create a test image file
        let test_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_image.png");

        // Create a simple test PNG image using the image crate
        let imgbuf = image::ImageBuffer::from_fn(32, 32, |x, y| {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            if dx * dx + dy * dy < 16.0 * 16.0 {
                image::Rgb([0u8, 0u8, 255u8]) // Blue circle
            } else {
                image::Rgb([255u8, 255u8, 255u8]) // White background
            }
        });
        imgbuf
            .save(&test_image_path)
            .expect("Failed to create test image");

        let params = json!({
            "mode": "add_image",
            "image_path": test_image_path.to_str().unwrap(),
            "width": 100,
            "height": 100,
            "style": {
                "alignment": "center"
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some("Image Caption"),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX image addition should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Clean up
        fs::remove_file(test_output_path).unwrap();
        fs::remove_file(test_image_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_invalid_path() {
        let result = docx_tool("nonexistent.docx", "extract_text", None, None).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_docx_invalid_operation() {
        let test_docx_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/sample.docx");

        let result = docx_tool(
            test_docx_path.to_str().unwrap(),
            "invalid_operation",
            None,
            None,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid operation");
    }

    #[tokio::test]
    async fn test_docx_update_without_content() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_output.docx");

        let result = docx_tool(test_output_path.to_str().unwrap(), "update_doc", None, None).await;

        assert!(result.is_err(), "Should fail without content");
    }

    #[tokio::test]
    async fn test_docx_update_preserve_content() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_preserve.docx");

        // First create a document with initial content
        let initial_content =
            "Initial content\nThis is the first paragraph.\nThis should stay in the document.";
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(initial_content),
            None,
        )
        .await;
        assert!(result.is_ok(), "Initial document creation should succeed");

        // Now append new content
        let new_content = "New content\nThis is an additional paragraph.";
        let params = json!({
            "mode": "append",
            "style": {
                "bold": true
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(new_content),
            Some(&params),
        )
        .await;
        assert!(result.is_ok(), "Content append should succeed");

        // Verify both old and new content exists
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();

        // Check for initial content
        assert!(
            text.contains("Initial content"),
            "Should contain initial content"
        );
        assert!(
            text.contains("first paragraph"),
            "Should contain first paragraph"
        );
        assert!(
            text.contains("should stay in the document"),
            "Should preserve existing content"
        );

        // Check for new content
        assert!(text.contains("New content"), "Should contain new content");
        assert!(
            text.contains("additional paragraph"),
            "Should contain appended paragraph"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }
}
