import test from 'ava';
import sinon from 'sinon';

const addEventListener = sinon.fake(((
    type: string,
    listener: (ev: MessageEvent) => void,
    options?: AddEventListenerOptions | boolean,
) => {
    if (type === 'message') {
        callback = listener;
    }
}) as typeof globalThis.addEventListener);
globalThis.addEventListener = addEventListener;

const postMessage = sinon.stub();
globalThis.postMessage = postMessage;

let callback: ((ev: MessageEvent) => void) | undefined;

test.before('load', async (t) => {
    await import('#compiler/worker');
    await new Promise((resolve) => setTimeout(resolve, 200)); // Wait for async initialization
    t.true(addEventListener.calledOnceWith('message', sinon.match.func));
    t.true(postMessage.calledOnceWith('ready'));
});

test.beforeEach(() => {
    addEventListener.resetHistory();
    postMessage.resetHistory();
});

test.serial('bad message', async (t) => {
    callback!(new MessageEvent('message', { data: 'bad message' }));
    callback!(new MessageEvent('message', { data: ['bad message'] }));
    callback!(new MessageEvent('message', { data: [0] }));
    await new Promise((resolve) => setTimeout(resolve, 10));
    t.false(postMessage.calledOnce);
});

test.serial('compile', async (t) => {
    callback!(new MessageEvent('message', { data: [0, 'nil', {}] }));
    await new Promise((resolve) => setTimeout(resolve, 10));
    t.true(
        postMessage.calledOnceWith([0, sinon.match.string, sinon.match.instanceOf(Uint32Array)], {
            transfer: [sinon.match.instanceOf(ArrayBuffer)],
        }),
    );
});
test.serial('compile error', async (t) => {
    callback!(new MessageEvent('message', { data: [1, ''] }));
    await new Promise((resolve) => setTimeout(resolve, 10));
    t.true(postMessage.calledOnceWith([1, sinon.match.instanceOf(Error)]));
});

test.serial('compile syntax error', async (t) => {
    callback!(new MessageEvent('message', { data: [2, '1 + ', {}] }));
    await new Promise((resolve) => setTimeout(resolve, 10));
    t.true(
        postMessage.calledOnceWith([2, undefined, sinon.match.instanceOf(Uint32Array)], {
            transfer: [sinon.match.instanceOf(ArrayBuffer)],
        }),
    );
});

test.serial('arg error', async (t) => {
    callback!(
        new MessageEvent('message', {
            data: [
                1,
                '',
                {
                    get input_mode() {
                        // eslint-disable-next-line @typescript-eslint/only-throw-error
                        throw 0;
                    },
                },
            ],
        }),
    );
    await new Promise((resolve) => setTimeout(resolve, 10));
    t.true(postMessage.calledOnceWith([1, sinon.match.instanceOf(Error).and(sinon.match.has('message', '0'))]));
});
