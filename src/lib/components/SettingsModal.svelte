<script lang="ts">
    import type { Settings } from '$lib/types';

    interface Props {
        open: boolean;
        settings: Settings;
        onsave?: (settings: Settings) => Promise<void> | void;
        onopenimport?: () => void;
        onopenexport?: () => void;
        onclose?: () => void;
    }

    let { open, settings, onsave, onopenimport, onopenexport, onclose }: Props = $props();
    let saving = $state(false);
    let hotkeyModifier = $state('Ctrl+Shift');
    let hotkeyKey = $state('V');
    let hotkeyError = $state('');

    const modifierOptions = [
        { value: '', label: '无修饰键' },
        { value: 'Ctrl', label: 'Ctrl' },
        { value: 'Alt', label: 'Alt' },
        { value: 'Shift', label: 'Shift' },
        { value: 'Ctrl+Alt', label: 'Ctrl + Alt' },
        { value: 'Ctrl+Shift', label: 'Ctrl + Shift' },
        { value: 'Alt+Shift', label: 'Alt + Shift' },
        { value: 'Ctrl+Alt+Shift', label: 'Ctrl + Alt + Shift' }
    ] as const;

    let draft = $state<Settings>({
        hotkey_modifiers: 0,
        hotkey_key: 0,
        hotkey: 'Ctrl+Shift+V',
        theme: 'system',
        keep_days: 30,
        max_records: 500,
        menu_width: 400,
        menu_height: 500,
        auto_start: false
    });

    function normalizeModifiers(values: string[]): string {
        const hasCtrl = values.includes('Ctrl');
        const hasAlt = values.includes('Alt');
        const hasShift = values.includes('Shift');
        const normalized = [
            hasCtrl ? 'Ctrl' : '',
            hasAlt ? 'Alt' : '',
            hasShift ? 'Shift' : ''
        ].filter(Boolean);
        return normalized.join('+');
    }

    function parseHotkey(hotkey: string): { modifier: string; key: string } {
        const tokens = hotkey
            .split('+')
            .map((t) => t.trim())
            .filter(Boolean);

        if (tokens.length === 0) {
            return { modifier: 'Ctrl+Shift', key: 'V' };
        }

        const key = normalizeKeyToken(tokens[tokens.length - 1]);
        const modTokens = tokens.slice(0, -1).map((t) => {
            const upper = t.toUpperCase();
            if (upper === 'CTRL' || upper === 'CONTROL') return 'Ctrl';
            if (upper === 'ALT' || upper === 'OPTION') return 'Alt';
            if (upper === 'SHIFT') return 'Shift';
            return '';
        }).filter(Boolean);

        const modifier = normalizeModifiers(modTokens);
        const hasOption = modifierOptions.some((o) => o.value === modifier);
        return {
            modifier: hasOption ? modifier : '',
            key: key || 'V'
        };
    }

    function normalizeKeyToken(raw: string): string {
        const token = raw.trim();
        if (!token) return '';

        if (/^[a-zA-Z0-9]$/.test(token)) {
            return token.toUpperCase();
        }

        const upper = token.toUpperCase();
        if (/^KEY[A-Z]$/.test(upper)) {
            return upper.slice(3);
        }
        if (/^DIGIT[0-9]$/.test(upper)) {
            return upper.slice(5);
        }
        if (/^F([1-9]|1[0-9]|2[0-4])$/.test(upper)) {
            return upper;
        }
        if (/^NUMPAD[0-9]$/.test(upper)) {
            return `Numpad${upper.slice(6)}`;
        }

        switch (upper) {
            case 'ESCAPE':
            case 'ESC':
                return 'Esc';
            case ' ':
            case 'SPACE':
            case 'SPACEBAR':
                return 'Space';
            case 'ARROWUP':
                return 'ArrowUp';
            case 'ARROWDOWN':
                return 'ArrowDown';
            case 'ARROWLEFT':
                return 'ArrowLeft';
            case 'ARROWRIGHT':
                return 'ArrowRight';
            default:
                return token;
        }
    }

    function keyLabel(token: string): string {
        if (/^Numpad[0-9]$/.test(token)) {
            return token.replace('Numpad', 'Num');
        }
        return token;
    }

    function formatHotkey(modifier: string, key: string): string {
        if (!modifier) return key;
        return `${modifier}+${key}`;
    }

    function keyTokenFromEvent(e: KeyboardEvent): string {
        if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
            return '';
        }

        if (/^Key[A-Z]$/.test(e.code)) {
            return e.code.slice(3);
        }

        if (/^Digit[0-9]$/.test(e.code)) {
            return e.code.slice(5);
        }

        if (/^Numpad[0-9]$/.test(e.code)) {
            return e.code;
        }

        if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(e.code)) {
            return e.code.toUpperCase();
        }

        if (/^[a-zA-Z0-9]$/.test(e.key)) {
            return e.key.toUpperCase();
        }

        switch (e.key) {
            case 'ArrowUp': return 'ArrowUp';
            case 'ArrowDown': return 'ArrowDown';
            case 'ArrowLeft': return 'ArrowLeft';
            case 'ArrowRight': return 'ArrowRight';
            case 'Enter': return 'Enter';
            case 'Tab': return 'Tab';
            case 'Escape': return 'Esc';
            case 'Backspace': return 'Backspace';
            case 'Delete': return 'Delete';
            case 'Insert': return 'Insert';
            case 'Home': return 'Home';
            case 'End': return 'End';
            case 'PageUp': return 'PageUp';
            case 'PageDown': return 'PageDown';
            case ' ': return 'Space';
            default: return normalizeKeyToken(e.key);
        }
    }

    function updateDraftHotkey() {
        draft.hotkey = formatHotkey(hotkeyModifier, hotkeyKey);
    }

    function closeModal() {
        onclose?.();
    }

    function handleHotkeyModifierChange(e: Event) {
        hotkeyModifier = (e.target as HTMLSelectElement).value;
        hotkeyError = '';
        updateDraftHotkey();
    }

    function handleHotkeyKeydown(e: KeyboardEvent) {
        e.preventDefault();
        const token = keyTokenFromEvent(e);
        if (!token) return;
        hotkeyKey = token;
        hotkeyError = '';
        updateDraftHotkey();
    }

    $effect(() => {
        if (open) {
            const parsed = parseHotkey(settings.hotkey);
            draft = {
                ...settings,
                hotkey: formatHotkey(parsed.modifier, parsed.key)
            };
            hotkeyModifier = parsed.modifier;
            hotkeyKey = parsed.key;
            hotkeyError = '';
        }
    });

    async function handleSubmit(e: Event) {
        e.preventDefault();
        if (!onsave || saving) return;
        if (!hotkeyKey.trim()) {
            hotkeyError = '请按下一个主键';
            return;
        }

        updateDraftHotkey();
        saving = true;
        try {
            await onsave({ ...draft });
            closeModal();
        } finally {
            saving = false;
        }
    }

    function handleBackdropClick(e: MouseEvent) {
        if (e.target === e.currentTarget) {
            closeModal();
        }
    }

    function handleBackdropKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            closeModal();
        }
    }
</script>

{#if open}
    <div
        class="settings-backdrop"
        role="button"
        tabindex="0"
        onclick={handleBackdropClick}
        onkeydown={handleBackdropKeydown}
    >
        <form class="settings-modal" onsubmit={handleSubmit}>
            <div class="modal-header">
                <h2>设置</h2>
                <button type="button" class="icon-btn" onclick={closeModal} aria-label="关闭设置">
                    ×
                </button>
            </div>

            <div class="modal-body">
                <label>
                    <span>呼出快捷键</span>
                    <div class="hotkey-row">
                        <select value={hotkeyModifier} oninput={handleHotkeyModifierChange}>
                            {#each modifierOptions as option}
                                <option value={option.value}>{option.label}</option>
                            {/each}
                        </select>
                        <input
                            type="text"
                            class="hotkey-input"
                            value={keyLabel(hotkeyKey)}
                            placeholder="按下按键"
                            readonly
                            onkeydown={handleHotkeyKeydown}
                        />
                    </div>
                    <small>示例：Ctrl + Shift + V（点击右侧输入框后按键）</small>
                    {#if hotkeyError}
                        <small class="error">{hotkeyError}</small>
                    {/if}
                </label>

                <label>
                    <span>主题</span>
                    <select bind:value={draft.theme}>
                        <option value="system">跟随系统</option>
                        <option value="light">浅色</option>
                        <option value="dark">深色</option>
                    </select>
                </label>

                <label>
                    <span>历史保留天数</span>
                    <input type="number" min="1" max="3650" bind:value={draft.keep_days} />
                </label>

                <label>
                    <span>最大记录数</span>
                    <input type="number" min="50" max="10000" step="50" bind:value={draft.max_records} />
                </label>

                <label>
                    <span>收藏导入导出</span>
                    <div class="transfer-row">
                        <button type="button" class="ghost-btn" onclick={() => onopenimport?.()}>
                            导入收藏(JSON)
                        </button>
                        <button type="button" class="ghost-btn" onclick={() => onopenexport?.()}>
                            导出收藏(JSON)
                        </button>
                    </div>
                    <small>导出时先选择目录，自动生成 JSON 文件；导入为增量模式，自动跳过重复收藏。</small>
                </label>

            </div>

            <div class="modal-footer">
                <button type="button" class="ghost-btn" onclick={closeModal}>取消</button>
                <button type="submit" class="primary-btn" disabled={saving}>
                    {saving ? '保存中...' : '保存'}
                </button>
            </div>
        </form>
    </div>
{/if}

<style>
    .settings-backdrop {
        position: fixed;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 6px;
        background: rgba(17, 24, 39, 0.45);
        z-index: 10;
    }

    .settings-modal {
        /* 主窗口最小 300x400 时保持可用，内容区只允许纵向滚动 */
        width: min(92vw, 360px);
        min-width: 272px;
        max-width: 100%;
        height: min(360px, calc(100vh - 12px));
        min-height: 320px;
        display: flex;
        flex-direction: column;
        background: var(--bg-primary);
        border: 1px solid var(--border-color);
        border-radius: 12px;
        box-shadow: 0 16px 40px rgba(0, 0, 0, 0.18);
        overflow: hidden;
    }

    .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 12px;
        border-bottom: 1px solid var(--border-color);
    }

    .modal-header h2 {
        margin: 0;
        font-size: 15px;
        color: var(--text-primary);
    }

    .icon-btn {
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: var(--text-secondary);
        cursor: pointer;
        font-size: 18px;
        line-height: 1;
    }

    .icon-btn:hover {
        background: var(--bg-hover);
    }

    .modal-body {
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        gap: 8px;
        padding: 10px 12px;
        overflow-y: auto;
        overflow-x: hidden;
    }

    label {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    label span {
        font-size: 12px;
        color: var(--text-secondary);
    }

    label small {
        font-size: 10px;
        color: var(--text-tertiary);
        white-space: normal;
        overflow-wrap: anywhere;
    }

    .error {
        color: var(--danger-color);
    }

    input,
    select {
        width: 100%;
        min-width: 0;
        height: 32px;
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 0 8px;
        font-size: 12px;
        color: var(--text-primary);
        background: var(--bg-primary);
        outline: none;
    }

    input:focus,
    select:focus {
        border-color: var(--accent-color);
    }

    .hotkey-row {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 6px;
        align-items: center;
    }

    .hotkey-row select,
    .hotkey-row .hotkey-input {
        height: 30px;
        padding: 0 6px;
        font-size: 11px;
    }

    .hotkey-input {
        text-align: center;
        letter-spacing: 0.2px;
    }

    .transfer-row {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 6px;
    }

    .modal-footer {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
        padding: 10px 12px;
        border-top: 1px solid var(--border-color);
    }

    .ghost-btn,
    .primary-btn {
        height: 32px;
        padding: 0 12px;
        border-radius: 8px;
        border: 1px solid var(--border-color);
        background: var(--bg-primary);
        color: var(--text-primary);
        cursor: pointer;
        font-size: 13px;
    }

    .primary-btn {
        border-color: var(--accent-color);
        background: var(--accent-color);
        color: #fff;
    }

    .ghost-btn:disabled,
    .primary-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
