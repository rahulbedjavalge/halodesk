<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { readText, writeText } from '@tauri-apps/api/clipboard';
  import { appWindow, PhysicalSize } from '@tauri-apps/api/window';

  type Role = 'system' | 'user' | 'assistant';
  type Message = { role: Role; content: string };
  type ImageData = { mime: string; base64: string };
  type AppConfig = {
    text_default_model: string;
    vision_default_model: string;
    fallback_model: string;
    models: { id: string; label: string; capability: string }[];
  };

  const presets = [
    {
      id: 'linkedin-comment',
      name: 'LinkedIn Comment',
      systemPrompt: 'Write 3 professional, thoughtful LinkedIn comments. Keep each under 40 words.'
    },
    {
      id: 'rewrite-cleaner',
      name: 'Rewrite Cleaner',
      systemPrompt: 'Rewrite the text to be clearer and more polished. Provide 1 improved and 1 shorter version.'
    },
    {
      id: 'grammar-fix',
      name: 'Grammar Fix',
      systemPrompt: 'Fix grammar and keep the original tone.'
    },
    {
      id: 'summarize',
      name: 'Summarize',
      systemPrompt: 'Summarize in 3 to 5 bullets.'
    },
    {
      id: 'summarize-screenshot',
      name: 'Summarize Screenshot',
      systemPrompt: 'Summarize the screenshot content in clear bullets.'
    },
    {
      id: 'translate',
      name: 'Translate',
      systemPrompt: 'Translate to plain English. Preserve meaning.'
    }
  ];

  const refineOptions = [
    { id: 'shorter', label: 'Shorter', instruction: 'Make it shorter.' },
    { id: 'direct', label: 'More direct', instruction: 'Make it more direct.' },
    { id: 'friendly', label: 'More friendly', instruction: 'Make it more friendly.' },
    { id: 'formal', label: 'More formal', instruction: 'Make it more formal.' },
    { id: 'emoji', label: 'Add emojis', instruction: 'Add tasteful emojis where appropriate.' }
  ];

  let port = 0;
  let prompt = '';
  let output = '';
  let isStreaming = false;
  let error = '';
  let activePreset = presets[0];
  let lastPrompt = '';
  let image: ImageData | null = null;
  let settingsOpen = false;
  let keySet = false;
  let activeModel = '';

  let defaultModel = '';
  let openrouterKey = '';

  const emptyConfig: AppConfig = {
    text_default_model: '',
    vision_default_model: '',
    fallback_model: '',
    models: []
  };

  let resizing = false;
  let resizeStart = { x: 0, y: 0 };
  let resizeStartSize = { width: 0, height: 0 };

  function hydrateConfig(config: AppConfig) {
    defaultModel = config.text_default_model || config.vision_default_model || '';
  }

  onMount(async () => {
    try {
      port = await invoke('router_port');
    } catch (err) {
      error = `Router port not available: ${String(err)}`;
    }

    try {
      const cfg = await invoke<AppConfig>('get_config');
      hydrateConfig(cfg ?? emptyConfig);
    } catch (err) {
      error = `Config load failed: ${String(err)}`;
    }

    try {
      keySet = await invoke('has_openrouter_key');
    } catch {
      keySet = false;
    }

    try {
      const clip = await readText();
      if (!prompt && clip && clip.trim().length > 0) {
        prompt = clip.trim();
      }
    } catch {
      // ignore clipboard errors
    }
  });

  function buildMessages(userText: string): Message[] {
    const msgs: Message[] = [];
    if (activePreset.systemPrompt) {
      msgs.push({ role: 'system', content: activePreset.systemPrompt });
    }
    msgs.push({ role: 'user', content: userText });
    return msgs;
  }

  async function sendChat(messages: Message[], imageData: ImageData | null) {
    if (!port) {
      error = 'Router port missing.';
      return;
    }

    const body = {
      preset_id: activePreset.id,
      messages,
      image: imageData,
      model_override: null,
      stream: true
    };

    const url = `http://127.0.0.1:${port}/v1/chat`;

    isStreaming = true;
    error = '';
    activeModel = '';

    try {
      const resp = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
      });

      if (!resp.ok || !resp.body) {
        const text = await resp.text();
        let message = text || `Request failed (${resp.status})`;
        try {
          const parsed = JSON.parse(text);
          if (parsed?.error) {
            message = parsed.error;
            if (parsed?.code) {
              message = `${message} (${parsed.code})`;
            }
          }
        } catch {
          // non-JSON error body
        }
        throw new Error(message);
      }

      output = '';
      await streamSse(resp, (event, data) => {
        if (event === 'delta') {
          if (typeof data?.text === 'string') {
            output += data.text;
          }
        } else if (event === 'meta') {
          activeModel = `${data?.provider ?? ''} ${data?.model ?? ''}`.trim();
        } else if (event === 'done') {
          if (data?.error) {
            error = String(data.error);
          } else if (data?.finish_reason === 'error') {
            error = 'Model returned an error.';
          }
        }
      });
    } catch (err) {
      error = String(err);
    } finally {
      isStreaming = false;
    }
  }

  async function send() {
    if (!prompt.trim()) return;
    if (!keySet) {
      error = 'OpenRouter key missing. Open Settings and add your key.';
      return;
    }
    if (!defaultModel.trim()) {
      error = 'Default model missing. Open Settings and add a model.';
      return;
    }
    lastPrompt = prompt.trim();
    await sendChat(buildMessages(lastPrompt), image);
  }

  async function regenerate() {
    if (!lastPrompt) return;
    await sendChat(buildMessages(lastPrompt), image);
  }

  async function refine(instruction: string) {
    if (!output) return;
    const msgs = buildMessages(lastPrompt || prompt);
    msgs.push({ role: 'assistant', content: output });
    msgs.push({ role: 'user', content: instruction });
    await sendChat(msgs, image);
  }

  async function copyOutput() {
    if (!output) return;
    await writeText(output);
  }

  async function captureScreen() {
    error = '';
    try {
      image = await invoke<ImageData>('capture_primary_display');
    } catch (err) {
      const message = String(err);
      if (/permission|denied|ScreenCapture/i.test(message)) {
        error = 'Capture failed: Screen recording permission is required.';
      } else if (/no screens found/i.test(message)) {
        error = 'Capture failed: No displays detected.';
      } else {
        error = `Capture failed: ${message}`;
      }
    }
  }

  function clearImage() {
    image = null;
  }

  async function saveSettings() {
    const modelId = defaultModel.trim();
    const config: AppConfig = {
      text_default_model: modelId,
      vision_default_model: modelId,
      fallback_model: modelId,
      models: modelId
        ? [
            { id: modelId, label: modelId, capability: 'text' },
            { id: modelId, label: modelId, capability: 'vision' }
          ]
        : []
    };

    try {
      await invoke('set_config', { config });
      if (openrouterKey.trim()) {
        await invoke('set_openrouter_key', { key: openrouterKey.trim() });
        keySet = true;
        openrouterKey = '';
      }
      settingsOpen = false;
    } catch (err) {
      error = `Settings save failed: ${String(err)}`;
    }
  }

  async function streamSse(
    resp: Response,
    onEvent: (event: string, data: any) => void
  ) {
    const reader = resp.body?.getReader();
    if (!reader) return;

    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      if (buffer.includes('\r\n')) {
        buffer = buffer.replace(/\r\n/g, '\n');
      }

      let boundary = buffer.indexOf('\n\n');
      while (boundary !== -1) {
        const chunk = buffer.slice(0, boundary);
        buffer = buffer.slice(boundary + 2);
        handleChunk(chunk, onEvent);
        boundary = buffer.indexOf('\n\n');
      }
    }
  }

  function handleChunk(chunk: string, onEvent: (event: string, data: any) => void) {
    let event = 'message';
    let data = '';

    for (const line of chunk.split('\n')) {
      if (line.startsWith('event:')) {
        event = line.slice(6).trim();
      } else if (line.startsWith('data:')) {
        data += line.slice(5).trim();
      }
    }

    if (!data) return;

    try {
      const payload = JSON.parse(data);
      onEvent(event, payload);
    } catch {
      onEvent(event, { text: data });
    }
  }

  function handlePromptKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  async function startDrag(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (target.closest('button') || target.closest('input') || target.closest('select')) {
      return;
    }
    await appWindow.startDragging();
  }

  function handleDragKeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    startDrag(event as unknown as MouseEvent);
  }

  async function hideWindow() {
    await appWindow.hide();
  }

  async function startResize(event: MouseEvent) {
    event.preventDefault();
    resizing = true;
    resizeStart = { x: event.screenX, y: event.screenY };
    const size = await appWindow.innerSize();
    resizeStartSize = { width: size.width, height: size.height };
    window.addEventListener('mousemove', resizeMove);
    window.addEventListener('mouseup', stopResize);
  }

  function handleResizeKeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    startResize(event as unknown as MouseEvent);
  }

  function resizeMove(event: MouseEvent) {
    if (!resizing) return;
    const dx = event.screenX - resizeStart.x;
    const dy = event.screenY - resizeStart.y;
    const width = Math.max(520, resizeStartSize.width + dx);
    const height = Math.max(420, resizeStartSize.height + dy);
    appWindow.setSize(new PhysicalSize(width, height));
  }

  function stopResize() {
    resizing = false;
    window.removeEventListener('mousemove', resizeMove);
    window.removeEventListener('mouseup', stopResize);
  }
</script>

<svelte:head>
  <title>HaloDesk</title>
</svelte:head>

<div class="stage">
  <div class="shell">
    <header
      class="topbar"
      role="button"
      aria-label="Drag window"
      tabindex="0"
      on:mousedown={startDrag}
      on:keydown={handleDragKeydown}
      data-tauri-drag-region
    >
      <div class="brand" data-tauri-drag-region>
        <div class="logo">H</div>
        <div>
          <div class="title">HaloDesk</div>
          <div class="subtitle">Floating AI overlay</div>
        </div>
      </div>
      <div class="top-actions">
        <button class="ghost" on:click={() => (settingsOpen = true)}>Settings</button>
        <button class="primary" on:click={send} disabled={isStreaming || !prompt.trim()}>Send</button>
        <button class="close" on:click={hideWindow} aria-label="Hide">×</button>
      </div>
    </header>

    <div class="preset-row">
      {#each presets as preset}
        <button class:active={preset.id === activePreset.id} on:click={() => (activePreset = preset)}>
          {preset.name}
        </button>
      {/each}
    </div>

    <div class="body">
      <div class="input-block">
        <div class="input-header">
          <div class="section-title">Prompt</div>
          <div class="row-actions">
            <button class="ghost" on:click={captureScreen} disabled={isStreaming}>Capture</button>
            <button class="ghost" on:click={() => (prompt = '')} disabled={isStreaming}>Clear</button>
            <button class="ghost" on:click={regenerate} disabled={!lastPrompt || isStreaming}>Regenerate</button>
            <button class="primary" on:click={send} disabled={isStreaming || !prompt.trim()}>Send</button>
          </div>
        </div>
        <textarea
          class="prompt"
          rows="5"
          bind:value={prompt}
          placeholder="Type or paste your request..."
          on:keydown={handlePromptKeydown}
        />
      </div>

      {#if image}
        <div class="image-preview">
          <img src={`data:${image.mime};base64,${image.base64}`} alt="Screenshot preview" />
          <button class="ghost" on:click={clearImage}>Remove</button>
        </div>
      {/if}

      <div class="output-block">
        <div class="output-header">
          <div class="section-title">Output</div>
          <button class="ghost" on:click={copyOutput} disabled={!output}>Copy</button>
        </div>
        <pre class="output">{output || (isStreaming ? 'Working...' : 'Your response will appear here.')}</pre>
      </div>
    </div>

    <div class="footer">
      <div class="status">
        <div>Router: {port ? `127.0.0.1:${port}` : 'starting...'}</div>
        <div>Key: {keySet ? 'set' : 'missing'}</div>
        {#if activeModel}
          <div>Model: {activeModel}</div>
        {/if}
      </div>
      <div class="chip-list">
        {#each refineOptions as chip}
          <button class="chip" on:click={() => refine(chip.instruction)} disabled={!output || isStreaming}>
            {chip.label}
          </button>
        {/each}
      </div>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div
      class="resize-handle"
      role="button"
      aria-label="Resize window"
      tabindex="0"
      on:mousedown={startResize}
      on:keydown={handleResizeKeydown}
    />
  </div>
</div>

{#if settingsOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <header>
        <h2>Settings</h2>
        <button class="ghost" on:click={() => (settingsOpen = false)}>Close</button>
      </header>

      <div class="modal-body">
        <label class="field">
          <span>OpenRouter API key</span>
          <input type="password" bind:value={openrouterKey} placeholder={keySet ? 'Key is set' : 'Enter key'} />
        </label>

        <label class="field">
          <span>Default model</span>
          <input type="text" bind:value={defaultModel} placeholder="openrouter:provider/model" />
        </label>
      </div>

      <footer>
        <button class="primary" on:click={saveSettings}>Save settings</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  @import url('https://fonts.googleapis.com/css2?family=Sora:wght@300;400;500;600;700&family=Fraunces:opsz,wght@9..144,600&display=swap');

  :global(body) {
    margin: 0;
    background: transparent;
    color: #0f1117;
    font-family: 'Sora', 'Segoe UI', sans-serif;
  }

  .stage {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 18px;
    background: transparent;
  }

  .shell {
    width: min(780px, 94vw);
    min-height: 520px;
    border-radius: 20px;
    background: linear-gradient(150deg, rgba(255, 255, 255, 0.95), rgba(245, 248, 252, 0.75));
    border: 1px solid rgba(15, 23, 42, 0.12);
    backdrop-filter: blur(18px);
    box-shadow: 0 28px 70px rgba(14, 18, 28, 0.28);
    overflow: hidden;
    position: relative;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 18px 12px;
    border-bottom: 1px solid rgba(15, 23, 42, 0.08);
    cursor: grab;
  }

  .topbar:active {
    cursor: grabbing;
  }

  .brand {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .logo {
    width: 38px;
    height: 38px;
    border-radius: 12px;
    background: linear-gradient(135deg, #ffd166, #06d6a0);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'Fraunces', serif;
    font-size: 18px;
    font-weight: 600;
  }

  .title {
    font-size: 18px;
    font-weight: 600;
  }

  .subtitle {
    font-size: 11px;
    opacity: 0.6;
  }

  .top-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .preset-row {
    display: flex;
    gap: 8px;
    padding: 12px 18px 8px;
    overflow-x: auto;
  }

  .preset-row button {
    border: 1px solid transparent;
    background: rgba(15, 23, 42, 0.06);
    padding: 8px 12px;
    border-radius: 12px;
    font-weight: 500;
    white-space: nowrap;
    cursor: pointer;
  }

  .preset-row button.active {
    background: rgba(6, 214, 160, 0.18);
    border-color: rgba(6, 214, 160, 0.5);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 8px 18px 16px;
  }

  .input-block,
  .output-block {
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid rgba(15, 23, 42, 0.1);
    border-radius: 16px;
    padding: 14px;
  }

  .input-header,
  .output-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 10px;
  }

  .row-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }

  .section-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.6;
  }

  .prompt {
    width: 100%;
    border: none;
    outline: none;
    background: transparent;
    resize: vertical;
    font-size: 15px;
    line-height: 1.5;
    min-height: 110px;
  }

  button {
    font-family: inherit;
  }

  .primary {
    background: #0f172a;
    color: white;
    border: none;
    padding: 8px 14px;
    border-radius: 10px;
    cursor: pointer;
  }

  .ghost {
    background: rgba(15, 23, 42, 0.08);
    border: none;
    padding: 7px 10px;
    border-radius: 10px;
    cursor: pointer;
  }

  .close {
    width: 28px;
    height: 28px;
    border-radius: 999px;
    border: 1px solid rgba(15, 23, 42, 0.12);
    background: rgba(255, 255, 255, 0.9);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }

  .image-preview {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(255, 255, 255, 0.92);
    border: 1px solid rgba(15, 23, 42, 0.1);
    border-radius: 14px;
    padding: 12px;
  }

  .image-preview img {
    max-width: 100%;
    border-radius: 12px;
  }

  .output {
    white-space: pre-wrap;
    font-size: 14px;
    line-height: 1.5;
    min-height: 120px;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 0 18px 16px;
    flex-wrap: wrap;
  }

  .status {
    font-size: 11px;
    opacity: 0.7;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .chip-list {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .chip {
    background: rgba(15, 23, 42, 0.08);
    border: none;
    border-radius: 999px;
    padding: 6px 12px;
    font-size: 11px;
    cursor: pointer;
  }

  .error {
    margin: 0 18px 16px;
    background: rgba(255, 111, 97, 0.2);
    color: #7a1f1f;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid rgba(122, 31, 31, 0.3);
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(6, 20, 35, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    width: min(820px, 90vw);
    background: white;
    border-radius: 18px;
    padding: 20px;
    box-shadow: 0 20px 60px rgba(15, 23, 42, 0.25);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .modal header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
  }

  .field input {
    padding: 8px 10px;
    border-radius: 10px;
    border: 1px solid rgba(15, 23, 42, 0.2);
  }

  .resize-handle {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 16px;
    height: 16px;
    border-right: 2px solid rgba(15, 23, 42, 0.3);
    border-bottom: 2px solid rgba(15, 23, 42, 0.3);
    transform: rotate(0deg);
    cursor: se-resize;
    opacity: 0.6;
  }

  @media (max-width: 720px) {
    .shell {
      min-height: 520px;
    }

    .row-actions {
      justify-content: flex-start;
    }
  }
</style>
