/** Ephemeral files go directly to the same-origin simulator, never to the archive. */
const $ = selector => document.querySelector(selector);
const MAX_BYTES = 128 * 1024 * 1024;
let current = null;
let dragDepth = 0;

function message(text, kind = 'loading') {
  if ($('#local-status').textContent !== text) $('#local-status').textContent = text;
  $('#local-status').dataset.state = kind;
}
function error(text) {
  $('#local-file-error').hidden = false;
  $('#local-file-error').textContent = text;
  if (current) current.inputError = text;
  if ($('#local-live').open) message(text, 'error');
}
function clearSession() {
  if (current) { clearTimeout(current.timer); current.frame.remove(); }
  current = null;
  $('#local-stage').replaceChildren();
}
function openFiles(files) {
  if (!files.length) return;
  if (files.length !== 1) { error('Choose one schematic at a time.'); return; }
  const file = files[0];
  if (!file.size) { error('That file is empty. Choose a saved schematic.'); return; }
  if (file.size > MAX_BYTES) { error('That file is larger than 128 MB. Try an individual build or a smaller selection.'); return; }
  $('#local-file-error').hidden = true;
  if ($('#detail').open) $('#detail').close();
  clearSession();
  const frame = document.createElement('iframe');
  frame.title = `Live workbench for ${file.name}`;
  frame.src = '/sim-lab/?embed=atlas&local=1';
  frame.allow = 'fullscreen';
  const session = { frame, file, requestId: crypto.randomUUID(), readyId: null, timer: null };
  current = session;
  $('#local-live-title').textContent = file.name;
  for (const id of ['local-tick', 'local-tps', 'local-entities']) $(`#${id}`).textContent = '—';
  $('#local-run-state').textContent = 'Loading';
  message('Opening the simulator…');
  $('#local-stage').replaceChildren(frame);
  if (!$('#local-live').open) $('#local-live').showModal();
  document.body.style.overflow = 'hidden';
  session.timer = setTimeout(() => {
    if (current === session && !session.readyId) message('The simulator did not start. Check the local server, then choose your file again.', 'error');
  }, 15000);
}

$('#local-drop').addEventListener('click', () => $('#local-file-input').click());
$('#local-replace').addEventListener('click', () => $('#local-file-input').click());
$('#local-file-input').addEventListener('change', event => {
  openFiles(event.target.files);
  event.target.value = ''; // selecting the same file again must reload it
});
$('#local-close').addEventListener('click', () => $('#local-live').close());
$('#local-live').addEventListener('close', () => {
  clearSession(); // removing the iframe stops its simulation and releases the file
  document.body.style.overflow = '';
  $('#local-drop').focus();
});

const isFiles = event => [...(event.dataTransfer?.types || [])].includes('Files');
document.addEventListener('dragenter', event => {
  if (!isFiles(event)) return;
  event.preventDefault(); dragDepth++;
  $('#local-drop').classList.add('drag-over');
});
document.addEventListener('dragover', event => {
  if (!isFiles(event)) return;
  event.preventDefault(); event.dataTransfer.dropEffect = 'copy';
});
document.addEventListener('dragleave', event => {
  if (!isFiles(event)) return;
  dragDepth = Math.max(0, dragDepth - 1);
  if (!dragDepth) $('#local-drop').classList.remove('drag-over');
});
document.addEventListener('drop', event => {
  if (!isFiles(event)) return;
  event.preventDefault(); dragDepth = 0; $('#local-drop').classList.remove('drag-over');
  openFiles(event.dataTransfer.files);
});
window.addEventListener('blur', () => { dragDepth = 0; $('#local-drop').classList.remove('drag-over'); });
window.addEventListener('message', event => {
  const session = current;
  if (!session || event.origin !== location.origin || event.source !== session.frame.contentWindow) return;
  if (event.data?.type === 'nucleation-atlas-ready') {
    const id = event.data.sessionId;
    if (typeof id !== 'string' || id === session.readyId) return;
    session.readyId = id; clearTimeout(session.timer);
    message(`Loading ${session.file.name}…`);
    session.frame.contentWindow.postMessage({ type: 'nucleation-atlas-command', command: 'load-file',
      file: session.file, requestId: session.requestId }, location.origin);
    return;
  }
  if (event.data?.type !== 'nucleation-atlas-state') return;
  const state = event.data.state;
  if (!state || state.localRequestId !== session.requestId) return;
  const kind = state.statusKind;
  if (session.inputError) message(session.inputError, 'error');
  else if (kind === 'error') message(state.statusMessage || 'This schematic could not be opened.', 'error');
  else if (kind === 'ready') {
    if (state.fileName) $('#local-live-title').textContent = state.fileName;
    message('Ready. Run, step, or click the controls in your build.', 'ready');
  } else message(state.statusMessage || 'Loading your schematic…');
  $('#local-tick').textContent = state.currentTick ?? '—';
  $('#local-tps').textContent = Number.isFinite(state.actualTps) ? Math.round(state.actualTps).toLocaleString() : '—';
  $('#local-entities').textContent = Number.isFinite(state.entities) ? state.entities.toLocaleString() : '—';
  $('#local-run-state').textContent = kind === 'ready' ? (state.running ? 'Running' : 'Paused') : kind === 'error' ? 'Error' : 'Loading';
});
