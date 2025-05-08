import test from 'ava'
import { writeFileSync, readFileSync } from 'fs'
import { join } from 'path'
import { fileURLToPath } from 'url'

// import { Transformer } from "@napi-rs/image";

import * as _ from '../index.js'

// test("clipboard image", async (t) => {
//   var c = _.clipboard();
//   const _buf = c.getImage();
//   if (!_buf) return;
//   const buf = Buffer.from(_buf.buffer);
//   console.log("loaded", buf.length);
//   const image = new Transformer(
//     // readFileSync(join(fileURLToPath(import.meta.url), "..", "test.png"))
//     buf
//   );
//   const { width, height, ...meta } = await image.metadata();
//   console.log({ width, height, ...meta });
//   // const rawPixels = await image.rawPixels();
//   // t.notThrows(() => {
//   //   var c = clipboard();
//   //   c.setImage(width, height, rawPixels);
//   // });
// });

// test("img", async (t) => {
// const clipboard = _.Clipboard.make();
// // const rawBuf = clipboard.readImage()
// const rawBuf = readFileSync(
//   join(fileURLToPath(import.meta.url), "..", "test.png")
// );
// t.assert(rawBuf !== null);
// clipboard.writeImage(rawBuf);
// const readBuf = clipboard.readImage();
// t.not(readBuf, null)
// t.deepEqual(rawBuf.length, readBuf?.length);
// console.log(buf);
// t.notThrows(() => {
//   clipboard.writeImage(readBuf);
// });
// });

// test("rw", async (t) => {
//   const clipboard = _.Clipboard.make();
//   console.log(clipboard.getAllKinds());
//   console.log("getText", JSON.stringify(clipboard.getText()));
//   console.log("readFiles", clipboard.readFiles());
//   const buf = clipboard.readImage();
//   t.assert(buf);
//   // console.log(buf);
//   t.notThrows(() => {
//     clipboard.writeImage(buf);
//   });
// });

test('img write', async (t) => {
  const clipboard = _.Clipboard.make()
  // // const rawBuf = clipboard.readImage()
  const rawBuf = readFileSync(join(fileURLToPath(import.meta.url), '..', 'test.png'))
  // t.assert(rawBuf !== null);
  // t.assert((await clipboard.writeImageAsync(rawBuf)) === true);
  // const readBuf = clipboard.readImage();
  // t.assert(readBuf !== null);
  // assert(readBuf !== null);
  const ctl = new AbortController()
  const promise = clipboard.writeImageAsync(rawBuf, ctl.signal)
  // // ctl.abort()
  // // console.log("abort")
  t.assert((await promise) === true)
  await t.notThrowsAsync(async () => {
    const readBuf2 = await clipboard.readImageAsync()
    // writeFileSync("./__test__/test3.png", readBuf);
    writeFileSync('./__test__/__output.png', readBuf2)
  })
  // console.log(rawBuf.length, readBuf.length, readBuf2.length);
})

// test("c write", async (t) => {
//   const clipboard = await _.checkClipboardImage();
//   console.log(clipboard);
//   t.assert(true);
// });

test('clipboard text', (t) => {
  const c = _.Clipboard.make()
  c.setText('😅')
  t.is(c.getText(), '😅')
})
