import test from 'ava'
import * as _ from '../index.js'

test('clipboard text', async (t) => {
  const clipboard = _.Clipboard.make()
  t.notThrows(() => clipboard.setText('😅'))
  t.is(await clipboard.getTextAsync(), '😅')
})
