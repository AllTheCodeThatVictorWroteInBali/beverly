function showFailure(message) {
  const loading = document.getElementById('loading-screen');
  loading.hidden = false;
  loading.textContent = `${message} Return to the website or reload to retry.`;
  loading.setAttribute('role', 'alert');
}
addEventListener('TrunkApplicationError', () => showFailure('Unable to start Beverly Agent in this browser.'));
addEventListener('error', () => showFailure('Beverly Agent could not run. The browser may not have enough memory.'));
addEventListener('unhandledrejection', () => showFailure('The demo failed to initialize.'));
addEventListener('keydown', event => {
  if (event.key === 'Escape' && parent !== window) {
    event.preventDefault();
    parent.postMessage({ type: 'beverly-agent:close' }, location.origin);
  }
}, true);