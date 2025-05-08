#![deny(clippy::all)]

use clipboard_rs::{
  common::RustImage, Clipboard, ClipboardContent, ClipboardContext, ContentFormat, RustImageData,
};
use napi::bindgen_prelude::*;

pub fn ctx() -> Option<ClipboardContext> {
  ClipboardContext::new().ok()
}

pub fn read_image(ctx: &ClipboardContext, kind: Option<ImageFormatKind>) -> Option<Buffer> {
  let img = ctx.get_image().ok()?;

  let png = (match kind.unwrap_or_default() {
    ImageFormatKind::Jpeg => img.to_jpeg(),
    #[cfg(target_os = "windows")]
    ImageFormatKind::Bmp => img.to_bitmap(),
    _ => img.to_png(),
  })
  .ok()?;

  let buf: Buffer = png.get_bytes().into();

  Some(buf)
}

pub fn write_image(ctx: &ClipboardContext, bytes: Buffer) -> Option<()> {
  let image_data = RustImageData::from_bytes(&bytes).ok()?;
  ctx.set_image(image_data).ok()
}

pub fn get_text(ctx: &ClipboardContext) -> Option<String> {
  ctx.get_text().ok()
}

pub fn get_files(ctx: &ClipboardContext) -> Option<Vec<String>> {
  ctx.get_files().ok()
}

pub fn get_html(ctx: &ClipboardContext) -> Option<String> {
  ctx.get_html().ok()
}

pub fn set_html(ctx: &ClipboardContext, text: String) -> Option<()> {
  ctx.set_html(text).ok()
}

#[derive(Default)]
#[napi]
pub enum ImageFormatKind {
  #[default]
  Png,
  Jpeg,
  Bmp,
}
#[derive(Debug, Clone)]
pub enum OutputContentFormat {
  Text,
  Rtf,
  Html,
  Image,
  Files,
  #[allow(unused)]
  Other(String),
}

pub fn to_output_format(content: &ClipboardContent) -> OutputContentFormat {
  match content {
    ClipboardContent::Text(_) => OutputContentFormat::Text,
    ClipboardContent::Rtf(_) => OutputContentFormat::Rtf,
    ClipboardContent::Html(_) => OutputContentFormat::Html,
    ClipboardContent::Image(_) => OutputContentFormat::Image,
    ClipboardContent::Files(_) => OutputContentFormat::Files,
    ClipboardContent::Other(format, _) => OutputContentFormat::Other(format.clone()),
  }
}

pub fn get_all_text(ctx: &ClipboardContext) -> Option<Vec<ClipboardContent>> {
  let contents: Vec<ClipboardContent> = ctx
    .get(&[
      ContentFormat::Text,
      ContentFormat::Html,
      ContentFormat::Image,
      ContentFormat::Files,
      ContentFormat::Rtf,
    ])
    .ok()?;
  Some(contents)
}

pub fn get_all_text_kind(ctx: &ClipboardContext) -> Option<Vec<OutputContentFormat>> {
  let contents: Vec<ClipboardContent> = get_all_text(ctx)?;
  let formats = contents.iter().map(to_output_format).collect();
  Some(formats)
}

pub fn set_files(ctx: &ClipboardContext, files: Vec<String>) -> Option<()> {
  ctx.set_files(files).ok()
}

pub fn set_text(ctx: &ClipboardContext, text: String) -> Option<()> {
  ctx.set_text(text).ok()
}

pub struct WriteTask {
  clipboard: crate::Clipboard,
  handle: Box<dyn FnMut(&crate::Clipboard) -> bool + Send>, //   pub img: Buffer,
}

impl WriteTask {
  pub fn new(
    clipboard: &crate::Clipboard,
    handle: Box<dyn FnMut(&crate::Clipboard) -> bool + Send>,
  ) -> WriteTask {
    // 创建异步任务

    WriteTask {
      clipboard: clipboard.clone(),
      handle,
    }
  }
}

#[napi]
impl Task for WriteTask {
  type Output = bool;
  type JsValue = bool;

  fn compute(&mut self) -> Result<Self::Output> {
    Ok(true)
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok((self.handle)(&self.clipboard))
  }
}
