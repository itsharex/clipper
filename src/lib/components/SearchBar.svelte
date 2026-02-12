<script lang="ts">
    interface Props {
        value?: string;
        onsubmit?: () => void;
        onchange?: (value: string) => void;
    }

    let { value = $bindable(''), onsubmit, onchange }: Props = $props();

    function handleInput(e: Event) {
        const target = e.target as HTMLInputElement;
        value = target.value;
        onchange?.(value);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && onsubmit) {
            onsubmit();
        }
    }

    function clear() {
        value = '';
        onchange?.('');
    }
</script>

<div class="search-bar">
    <div class="input-wrap">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.35-4.35" />
        </svg>
        <input
            type="text"
            placeholder="搜索剪贴板内容..."
            bind:value
            oninput={handleInput}
            onkeydown={handleKeydown}
            aria-label="搜索"
        />
        {#if value}
            <button type="button" class="clear-btn" onclick={clear} aria-label="清空搜索">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M18 6 6 18M6 6l12 12" />
                </svg>
            </button>
        {/if}
    </div>
</div>

<style>
    .search-bar {
        width: 100%;
    }

    .input-wrap {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        height: 36px;
        padding: 0 10px;
        border: 1px solid var(--border-color);
        border-radius: 8px;
        background: var(--bg-primary);
    }

    .input-wrap:focus-within {
        border-color: var(--accent-color);
        box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-color) 20%, transparent);
    }

    .input-wrap svg {
        width: 16px;
        height: 16px;
        color: var(--text-secondary);
        flex-shrink: 0;
    }

    input {
        flex: 1;
        min-width: 0;
        border: none;
        outline: none;
        background: transparent;
        color: var(--text-primary);
        font-size: 13px;
    }

    input::placeholder {
        color: var(--text-tertiary);
    }

    .clear-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        height: 24px;
        padding: 0;
        border: none;
        border-radius: 6px;
        background: transparent;
        cursor: pointer;
    }

    .clear-btn:hover {
        background: var(--bg-hover);
    }

    .clear-btn svg {
        width: 14px;
        height: 14px;
    }
</style>
