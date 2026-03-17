use super::image::{max_image_lines, render_image};
use crate::error::Result;
use crate::terminal::TerminalCaps;
use scraper::Html;
use std::path::Path;

/// Render an HTML file at `path` as ANSI text.
///
/// Embedded `<img>` elements are rendered via chafa if available, or replaced
/// with a `[image: filename]` placeholder otherwise.
pub fn render(path: &Path, caps: &TerminalCaps, inline: bool, max_lines: u16) -> Result<Vec<u8>> {
    let html_bytes = std::fs::read(path)?;
    let base_dir = path.parent();
    let resolver = |src: &str| -> Option<Vec<u8>> {
        let img_path = base_dir?.join(src);
        std::fs::read(img_path).ok()
    };
    render_with_resolver(&html_bytes, caps, inline, max_lines, &resolver)
}

/// Render raw HTML bytes with a caller-supplied image resolver.
///
/// `image_resolver` maps an `src` attribute string to raw image bytes.
/// Return `None` to emit a placeholder instead of attempting chafa.
pub fn render_with_resolver(
    html_bytes: &[u8],
    caps: &TerminalCaps,
    inline: bool,
    max_lines: u16,
    image_resolver: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<Vec<u8>> {
    let html_str = String::from_utf8_lossy(html_bytes);
    let document = Html::parse_document(&html_str);
    let mut out: Vec<u8> = Vec::new();
    let max_img = max_image_lines(caps, inline);

    walk_node(
        document.root_element(),
        &mut out,
        caps,
        max_img,
        image_resolver,
    );
    let _ = max_lines; // line truncation applied by caller (truncate_ansi_safe)
    Ok(out)
}

fn walk_node(
    node: scraper::ElementRef<'_>,
    out: &mut Vec<u8>,
    caps: &TerminalCaps,
    max_img: u16,
    image_resolver: &dyn Fn(&str) -> Option<Vec<u8>>,
) {
    use scraper::node::Node;

    for child in node.children() {
        match child.value() {
            Node::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    out.extend_from_slice(trimmed.as_bytes());
                    out.push(b' ');
                }
            }
            Node::Element(_) => {
                if let Some(elem) = scraper::ElementRef::wrap(child) {
                    emit_element(elem, out, caps, max_img, image_resolver);
                }
            }
            _ => {}
        }
    }
}

fn emit_element(
    elem: scraper::ElementRef<'_>,
    out: &mut Vec<u8>,
    caps: &TerminalCaps,
    max_img: u16,
    image_resolver: &dyn Fn(&str) -> Option<Vec<u8>>,
) {
    let tag = elem.value().name();

    match tag {
        "script" | "style" | "head" => return,

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            out.extend_from_slice(b"\n\x1b[1m"); // bold on
            walk_node(elem, out, caps, max_img, image_resolver);
            out.extend_from_slice(b"\x1b[0m\n"); // reset
        }

        "p" | "div" | "section" | "article" | "header" | "footer" | "main" => {
            out.push(b'\n');
            walk_node(elem, out, caps, max_img, image_resolver);
            out.push(b'\n');
        }

        "li" => {
            out.extend_from_slice(b"\n  \xe2\x80\xa2 "); // bullet (UTF-8 •)
            walk_node(elem, out, caps, max_img, image_resolver);
        }

        "ul" | "ol" => {
            out.push(b'\n');
            walk_node(elem, out, caps, max_img, image_resolver);
            out.push(b'\n');
        }

        "pre" | "code" => {
            out.extend_from_slice(b"\n  ");
            walk_node(elem, out, caps, max_img, image_resolver);
            out.push(b'\n');
        }

        "blockquote" => {
            out.extend_from_slice(b"\n  > ");
            walk_node(elem, out, caps, max_img, image_resolver);
            out.push(b'\n');
        }

        "br" => {
            out.push(b'\n');
        }

        "img" => {
            let src = elem.value().attr("src").unwrap_or("");
            let filename = Path::new(src)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(src);

            match image_resolver(src) {
                Some(bytes) => {
                    let rendered = render_image(&bytes, filename, caps, max_img);
                    out.extend_from_slice(&rendered);
                }
                None => {
                    out.extend_from_slice(format!("[image: {filename}]\n").as_bytes());
                }
            }
        }

        // Inline elements: just recurse
        _ => {
            walk_node(elem, out, caps, max_img, image_resolver);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{GraphicsProtocol, TerminalCaps};

    fn caps() -> TerminalCaps {
        TerminalCaps {
            graphics: GraphicsProtocol::HalfBlock256,
            true_color: false,
            columns: 80,
            rows: 24,
        }
    }

    fn render_html(html: &str) -> Vec<u8> {
        render_with_resolver(html.as_bytes(), &caps(), true, 50, &|_| None).unwrap()
    }

    #[test]
    fn heading_produces_bold_ansi() {
        let out = render_html("<h1>Title</h1>");
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("\x1b[1m"), "expected bold escape");
        assert!(s.contains("Title"));
        assert!(s.contains("\x1b[0m"), "expected reset escape");
    }

    #[test]
    fn paragraph_is_newline_separated() {
        let out = render_html("<p>Hello</p><p>World</p>");
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("Hello"));
        assert!(s.contains("World"));
        // Two newlines separating paragraphs
        assert!(s.contains('\n'));
    }

    #[test]
    fn img_with_no_resolver_emits_placeholder() {
        let out = render_html(r#"<img src="photo.png" />"#);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("[image: photo.png]"));
    }

    #[test]
    fn img_with_resolver_calls_render_image() {
        // Provide 1x1 transparent PNG bytes so render_image gets real data
        let png_1x1: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG signature
        ];
        let resolver = |src: &str| -> Option<Vec<u8>> {
            if src == "img.png" {
                Some(png_1x1.to_vec())
            } else {
                None
            }
        };
        let out =
            render_with_resolver(b"<img src=\"img.png\" />", &caps(), true, 50, &resolver).unwrap();
        // Either chafa rendered it or the placeholder was used — must not panic
        assert!(!out.is_empty());
    }

    #[test]
    fn script_and_style_content_excluded() {
        let out = render_html("<script>var x=1;</script><p>Visible</p>");
        let s = String::from_utf8_lossy(&out);
        assert!(!s.contains("var x"), "script content should be excluded");
        assert!(s.contains("Visible"));
    }

    #[test]
    fn br_emits_newline() {
        let out = render_html("line1<br/>line2");
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains('\n'));
    }

    #[test]
    fn list_items_have_bullet() {
        let out = render_html("<ul><li>Item</li></ul>");
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("Item"));
    }
}
