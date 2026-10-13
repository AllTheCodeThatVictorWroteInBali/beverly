let worker;
let events = [];
let busy = false;
let ready = false;
let loading = false;
let mobileInput;
let inputEvents = [];
let responseText = '';
let announcedProgress = -1;

function announcementRegion() {
  let region = document.getElementById('agent-announcement');
  if (!region) {
    region = document.createElement('p');
    region.id = 'agent-announcement';
    region.setAttribute('role', 'status');
    region.setAttribute('aria-live', 'polite');
    region.setAttribute('aria-atomic', 'true');
    region.style.cssText = 'position:fixed;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap;';
    document.body.append(region);
  }
  return region;
}

export function syncMobileInput(value, busy, left, top, width, height) {
  if (!matchMedia('(pointer: coarse)').matches) return;
  if (!mobileInput) {
    mobileInput = document.createElement('textarea');
    mobileInput.id = 'mobile-message-input';
    mobileInput.setAttribute('aria-label', 'Message Beverly Agent');
    mobileInput.setAttribute('enterkeyhint', 'send');
    mobileInput.autocomplete = 'off';
    mobileInput.style.cssText = 'position:fixed;z-index:2;margin:0;border:0;padding:12px;background:transparent;color:transparent;caret-color:transparent;resize:none;font:16px sans-serif;box-sizing:border-box;outline:none;';
    const changed = () => inputEvents.push({ type: 'input', value: mobileInput.value, selection: mobileInput.selectionStart });
    mobileInput.addEventListener('input', changed);
    mobileInput.addEventListener('select', changed);
    mobileInput.addEventListener('focus', () => inputEvents.push({ type: 'focus' }));
    mobileInput.addEventListener('blur', () => inputEvents.push({ type: 'blur' }));
    mobileInput.addEventListener('keydown', event => {
      if (event.key === 'Escape') return;
      event.stopPropagation();
      if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
        event.preventDefault();
        inputEvents.push({ type: 'submit' });
      }
    });
    mobileInput.addEventListener('keyup', event => { if (event.key !== 'Escape') event.stopPropagation(); });
    document.body.append(mobileInput);
  }
  mobileInput.style.display = width > 0 && height > 0 ? 'block' : 'none';
  mobileInput.style.left = `${left}px`;
  mobileInput.style.top = `${top}px`;
  mobileInput.style.width = `${width}px`;
  mobileInput.style.height = `${height}px`;
  mobileInput.readOnly = busy;
  if (!inputEvents.some(event => event.type === 'input') && mobileInput.value !== value) mobileInput.value = value;
}

export function drainMobileInput() {
  const result = JSON.stringify(inputEvents);
  inputEvents = [];
  return result;
}

export function initializeWorker() {
  if (worker) return;
  try {
    worker = new Worker(new URL('./inference/worker.js', location.href), { type: 'module' });
    worker.onmessage = ({ data }) => {
      const announcement = announcementRegion();
      if (data.type === 'status') announcement.textContent = data.stage;
      if (data.type === 'ready') announcement.textContent = 'TinyLlama is ready.';
      if (data.type === 'started') { responseText = ''; announcement.textContent = 'Generating a response locally.'; }
      if (data.type === 'token') responseText = data.text;
      if (data.type === 'done') announcement.textContent = `${data.cancelled ? 'Generation stopped. ' : ''}${responseText}`;
      if (data.type === 'error') announcement.textContent = data.message;
      if (data.type === 'progress') {
        const progress = Math.floor(data.fraction * 10);
        if (progress !== announcedProgress) {
          announcement.textContent = `${data.stage}: ${Math.round(data.fraction * 100)} percent.`;
          announcedProgress = progress;
        }
      }
      if (data.type === 'ready') {
        ready = true;
        loading = false;
        parent.postMessage({ type: 'beverly-agent:ready' }, location.origin);
      }
      if (data.type === 'started') busy = true;
      if (data.type === 'done' || data.type === 'error') busy = false;
      if (data.type === 'error') loading = false;
      events.push(data);
      if (data.type === 'error' && (!ready || data.fatal)) {
        worker.terminate(); worker = undefined; ready = false;
      }
      if (data.type === 'notice') {
        const notice = document.getElementById('platform-notice');
        notice.textContent = data.message;
        notice.hidden = false;
      }
    };
    worker.onerror = event => {
      events.push({ type: 'error', fatal: true, message: `The inference worker stopped: ${event.message || 'memory pressure or unsupported browser'}. Retry or return to the website.` });
      worker.terminate(); worker = undefined; busy = false; ready = false; loading = false;
    };
  } catch (error) { events.push({ type: 'error', message: String(error.message || error) }); }
}

export function sendCommand(message) {
  if (!worker) ready = false;
  initializeWorker();
  const command = JSON.parse(message);
  if (command.command === 'load') loading = true;
  worker?.postMessage(command);
}

export function drainEvents() {
  const result = JSON.stringify(events);
  events = [];
  return result;
}

addEventListener('message', ({ origin, data }) => {
  if (origin !== location.origin) return;
  if (data?.type === 'beverly-agent:visibility' && !data.visible && (busy || loading)) sendCommand('{"command":"cancel"}');
  if (data?.type === 'beverly-agent:escape') parent.postMessage({ type: 'beverly-agent:close' }, location.origin);
});
document.addEventListener('visibilitychange', () => { if (document.hidden && (busy || loading)) sendCommand('{"command":"cancel"}'); });