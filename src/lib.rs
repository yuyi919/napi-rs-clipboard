#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use base64::{engine::general_purpose, Engine as _};
use duct::cmd;
use napi::bindgen_prelude::Buffer;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
// use tokio::sync::{RwLock as AsyncRwLock};
// use std::thread::sleep;
// use std::time::Duration;

use napi::bindgen_prelude::*;
use napi::Status::GenericFailure;
mod clipboard;
// mod tasks;

/// Set the clipboard contents using OSC 52 (picked up by most terminals)
fn set_clipboard_osc_52(text: String) {
  print!("\x1B]52;c;{}\x07", general_purpose::STANDARD.encode(text));
}

/// Set the Windows clipboard using clip.exe in WSL
fn set_wsl_clipboard(s: String) -> Result<()> {
  let mut clipboard = Command::new("clip.exe")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
  {
    let mut clipboard_stdin = clipboard
      .stdin
      .take()
      .ok_or_else(|| Error::new(GenericFailure, "Could not get stdin handle for clip.exe"))?;
    clipboard_stdin.write_all(s.as_bytes())?;
  }

  clipboard
    .wait()
    .map_err(|err| {
      Error::new(
        GenericFailure,
        format!("Could not wait for clip.exe, reason: {err}"),
      )
    })
    .and_then(|status| {
      if status.success() {
        Ok(())
      } else {
        Err(Error::new(
          GenericFailure,
          format!("clip.exe stopped with status {status}"),
        ))
      }
    })?;

  Ok(())
}

#[derive(Clone)]
#[napi]
pub struct Clipboard {
  instance: Arc<RwLock<Option<clipboard_rs::ClipboardContext>>>,
}

#[napi]
impl Clipboard {
  // fn inner(&mut self) -> std::sync::RwLockWriteGuard<'_, Option<clipboard_rs::ClipboardContext>> {
  //   let mut s: std::sync::RwLockWriteGuard<'_, Option<clipboard_rs::ClipboardContext>> =
  //     self.lazy_inner.write().unwrap();
  //   if s.is_none() {
  //     println!("[clipboard] init ctx");
  //     *s = clipboard::ctx();
  //   };
  //   return s;
  // }

  fn inner_read(
    &self,
  ) -> Option<std::sync::RwLockReadGuard<'_, Option<clipboard_rs::ClipboardContext>>> {
    let guard = self.instance.read();
    if guard.is_ok() {
      let lock = guard.unwrap();
      if lock.is_some() {
        return Some(lock);
      }
      drop(lock);
    };
    let mut guard = self.instance.write().unwrap();
    *guard = clipboard::ctx();
    println!("[clipboard] init ctx");
    drop(guard);
    let guard = self.instance.read().unwrap();
    Some(guard)
  }

  pub fn try_read<U, F>(&self, f: F) -> Option<U>
  where
    F: FnOnce(&clipboard_rs::ClipboardContext) -> Option<U>,
  {
    let guard = self.inner_read()?;
    let ctx = guard.as_ref()?;
    f(ctx)
  }

  #[napi]
  pub fn make() -> Clipboard {
    Clipboard {
      instance: Arc::new(RwLock::new(None)),
    }
  }

  /// Copy text to the clipboard. Has special handling for WSL and SSH sessions, otherwise
  /// falls back to the cross-platform `clipboard` crate
  #[napi]
  pub fn set_text(&self, text: String) -> bool {
    if wsl::is_wsl() {
      set_wsl_clipboard(text).is_ok()
    } else if env::var("SSH_CLIENT").is_ok() {
      // we're in an SSH session, so set the clipboard using OSC 52 escape sequence
      set_clipboard_osc_52(text);
      return true;
    } else {
      // we're probably running on a host/primary OS, so use the default clipboard
      return self
        .try_read(|ctx| clipboard::set_text(ctx, text))
        .is_some();
    }
  }

  #[napi]
  pub fn get_text(&self) -> Option<String> {
    if wsl::is_wsl() {
      let stdout = cmd!("powershell.exe", "get-clipboard").read().ok()?;
      Some(stdout.trim().to_string())
    } else if env::var("SSH_CLIENT").is_ok() {
      // Err(Error::new(GenericFailure, "SSH clipboard not supported"))
      None
    } else {
      // we're probably running on a host/primary OS, so use the default clipboard
      let ctx = self.inner_read()?;
      let ctx = ctx.as_ref()?;
      clipboard::get_text(ctx)
    }
  }

  #[napi]
  pub fn read_files(&self) -> Option<Vec<String>> {
    let ctx = self.inner_read()?;
    let ctx = ctx.as_ref()?;
    clipboard::get_files(ctx)
  }

  #[napi]
  pub fn write_files(&self, files: Vec<String>) -> bool {
    self
      .try_read(|ctx| clipboard::set_files(ctx, files))
      .is_some()
  }

  #[napi]
  pub fn read_html(&self) -> Option<String> {
    let ctx = self.inner_read()?;
    let ctx = ctx.as_ref()?;
    clipboard::get_html(ctx)
  }

  #[napi]
  pub fn write_html(&self, html: String) -> bool {
    self
      .try_read(|ctx| clipboard::set_html(ctx, html))
      .is_some()
  }

  #[napi]
  pub fn get_all_kinds(&self) -> Option<Vec<String>> {
    let ctx = self.inner_read()?;
    let ctx = ctx.as_ref()?;
    let vec = clipboard::get_all_text_kind(ctx)?;

    Some(vec.iter().map(|a| format!("{:?}", a)).collect())
  }

  #[napi]
  pub fn read_image(&self, kind: Option<clipboard::ImageFormatKind>) -> Option<Buffer> {
    let ctx = self.inner_read()?;
    let ctx = ctx.as_ref()?;
    clipboard::read_image(ctx, kind)
  }

  #[napi]
  pub fn write_image(&self, img: Buffer) -> bool {
    self
      .try_read(|ctx| clipboard::write_image(ctx, img))
      .is_some()
  }

  #[napi]
  pub async fn read_image_async(&self, kind: Option<clipboard::ImageFormatKind>) -> Result<Buffer> {
    self
      .read_image(kind)
      .ok_or_else(|| Error::from_reason("fail"))
  }

  #[napi]
  pub fn write_image_async(
    &self,
    img: Buffer,
    signal: Option<AbortSignal>,
  ) -> AsyncTask<clipboard::WriteTask> {
    AsyncTask::with_optional_signal(
      clipboard::WriteTask::new(self, Box::new(move |ctx| ctx.write_image(img.clone()))),
      signal,
    )
  }

  // #[napi]
  // pub fn write_image_async(
  //   &self,
  //   img: Buffer,
  //   signal: Option<AbortSignal>,
  // ) -> AsyncTask<tasks::ClipboardTask> {
  //   // 创建异步任务
  //   let task = tasks::ClipboardTask::new(self.clone(), img);
  //   AsyncTask::with_optional_signal(task, signal)
  // }

  // #[napi]
  // pub async unsafe fn write_image_async3(&self, img: Buffer) -> Result<bool> {
  //   // println!("write_image");
  //   // 获取写锁以安全访问 ClipboardContext
  //   let guard = self.inner_read().unwrap();
  //   let ctx = guard.as_ref().unwrap();
  //   // 检查是否包含图像
  //   let has_image = clipboard::write_image(ctx, img).is_some();

  //   // println!("write_imageEnd");
  //   Ok(has_image)
  // }
}
