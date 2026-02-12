<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
    import { getVersion } from '@tauri-apps/api/app';
    import { open as openDialog } from '@tauri-apps/plugin-dialog';
    import { onMount } from 'svelte';
    import type { ClipboardRecord, Settings } from '$lib/types';
    import SearchBar from '$lib/components/SearchBar.svelte';
    import ClipboardList from '$lib/components/ClipboardList.svelte';
    import SettingsModal from '$lib/components/SettingsModal.svelte';

    type ExportFavoritesResult = {
        count: number;
        path: string;
    };

    let records = $state<ClipboardRecord[]>([]);
    let loading = $state(false);
    let searchKeyword = $state('');
    let favoritesOnly = $state(false);
    let searchTimeout: ReturnType<typeof setTimeout>;
    let refreshInterval: ReturnType<typeof setInterval>;
    let unlistenOpenSettings: UnlistenFn | null = null;
    let unlistenOpenAbout: UnlistenFn | null = null;
    let unlistenMainWindowOpened: UnlistenFn | null = null;
    let settingsOpen = $state(false);
    let aboutOpen = $state(false);
    let appVersion = $state('0.1.0');
    let clearConfirmOpen = $state(false);
    let addFavoriteOpen = $state(false);
    let favoriteInput = $state('');
    let addFavoriteSaving = $state(false);
    let settings = $state<Settings>({
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

    const DEFAULT_LIMIT = 50;

    function preApplyCachedTheme() {
        if (typeof window === 'undefined') return;
        const cached = window.localStorage.getItem('clipper-theme');
        if (cached === 'light' || cached === 'dark') {
            document.documentElement.setAttribute('data-theme', cached);
        } else if (cached === 'system') {
            document.documentElement.removeAttribute('data-theme');
        }
    }

    preApplyCachedTheme();

    function listCommand(): 'get_history_records' | 'get_favorite_records' {
        return favoritesOnly ? 'get_favorite_records' : 'get_history_records';
    }

    function searchCommand(): 'search_records' | 'search_favorite_records' {
        return favoritesOnly ? 'search_favorite_records' : 'search_records';
    }

    function pageTitle(): string {
        return favoritesOnly ? '收藏' : '历史记录';
    }

    function emptyTitle(): string {
        return favoritesOnly ? '暂无收藏' : '暂无历史记录';
    }

    function emptyHint(): string {
        return favoritesOnly ? '点击记录右侧星标即可收藏常用内容' : '复制内容后会自动记录';
    }

    function sortRecordsByPinnedAndTime(items: ClipboardRecord[]): ClipboardRecord[] {
        return [...items].sort((a, b) => {
            const pinDiff = Number(b.is_pinned) - Number(a.is_pinned);
            if (pinDiff !== 0) return pinDiff;
            return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
        });
    }

    function applyTheme(theme: Settings['theme']) {
        const root = document.documentElement;
        if (typeof window !== 'undefined') {
            window.localStorage.setItem('clipper-theme', theme);
        }
        if (theme === 'system') {
            root.removeAttribute('data-theme');
            return;
        }
        root.setAttribute('data-theme', theme);
    }

    async function loadHistory(showLoading: boolean = true) {
        try {
            if (showLoading) {
                loading = true;
            }
            const keyword = searchKeyword.trim();
            if (keyword) {
                records = await invoke<ClipboardRecord[]>(searchCommand(), {
                    keyword,
                    limit: DEFAULT_LIMIT
                });
            } else {
                records = await invoke<ClipboardRecord[]>(listCommand(), {
                    limit: DEFAULT_LIMIT,
                    offset: 0
                });
            }
        } catch (error) {
            console.error('Failed to load history:', error);
        } finally {
            if (showLoading) {
                loading = false;
            }
        }
    }

    // 静默刷新，不显示 loading
    async function refreshHistory() {
        try {
            if (settingsOpen || clearConfirmOpen || addFavoriteOpen) {
                return;
            }
            // 搜索模式下不覆盖搜索结果
            if (searchKeyword.trim()) {
                return;
            }
            const newRecords = await invoke<ClipboardRecord[]>(listCommand(), {
                limit: DEFAULT_LIMIT,
                offset: 0
            });
            // 静默更新，不触发 loading
            records = newRecords;
        } catch (error) {
            console.error('Failed to refresh:', error);
        }
    }

    async function searchHistory(keyword: string) {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(async () => {
            try {
                loading = true;
                if (keyword.trim()) {
                    records = await invoke<ClipboardRecord[]>(searchCommand(), {
                        keyword,
                        limit: DEFAULT_LIMIT
                    });
                } else {
                    records = await invoke<ClipboardRecord[]>(listCommand(), {
                        limit: DEFAULT_LIMIT,
                        offset: 0
                    });
                }
            } catch (error) {
                console.error('Failed to search:', error);
            } finally {
                loading = false;
            }
        }, 300);
    }

    async function handleSearch(value: string) {
        searchKeyword = value;
        await searchHistory(value);
    }

    async function resetSearchStateOnShow() {
        if (!searchKeyword.trim()) return;
        clearTimeout(searchTimeout);
        searchKeyword = '';
        await loadHistory(false);
    }

    async function loadSettings() {
        try {
            settings = await invoke<Settings>('get_app_settings');
            applyTheme(settings.theme);
        } catch (error) {
            console.error('Failed to load settings:', error);
        }
    }

    function saveSettings(nextSettings: Settings) {
        const previous = { ...settings };
        settingsOpen = false;
        settings = { ...nextSettings };
        applyTheme(settings.theme);
        void invoke('save_app_settings', { settings: nextSettings })
            .then(() => {
            void loadSettings();
            void loadHistory();
            })
            .catch((error) => {
                settings = previous;
                applyTheme(settings.theme);
                console.error('Failed to save settings:', error);
            });
    }

    async function handleCopy(id: number) {
        const record = records.find(r => r.id === id);
        if (record) {
            try {
                console.log('[clipper] paste start', { id: record.id, len: record.content?.length ?? 0 });
                await invoke('paste_record_content', { id: record.id });
                console.log('[clipper] paste command finished');
            } catch (error) {
                console.error('Failed to paste record content:', error);
            }
        }
    }

    async function handleDelete(id: number) {
        try {
            await invoke('delete_clipboard_record', { id });
            records = records.filter(r => r.id !== id);
        } catch (error) {
            console.error('Failed to delete:', error);
        }
    }

    async function handleFavorite(id: number, favorite: boolean) {
        const previous = records;
        if (favoritesOnly && !favorite) {
            records = records.filter((r) => r.id !== id);
        } else {
            records = records.map((r) =>
                r.id === id ? { ...r, is_favorite: favorite } : r
            );
        }

        try {
            await invoke('set_record_favorite_state', { id, favorite });
        } catch (error) {
            records = previous;
            console.error('Failed to update favorite state:', error);
        }
    }

    async function handlePinned(id: number, pinned: boolean) {
        const previous = records;
        records = records.map((r) =>
            r.id === id ? { ...r, is_pinned: pinned } : r
        );
        records = sortRecordsByPinnedAndTime(records);

        try {
            await invoke('set_record_pinned_state', { id, pinned });
        } catch (error) {
            records = previous;
            console.error('Failed to update pinned state:', error);
        }
    }

    function openAddFavoriteDialog() {
        addFavoriteOpen = true;
        favoriteInput = '';
    }

    function closeAddFavoriteDialog() {
        if (addFavoriteSaving) return;
        addFavoriteOpen = false;
    }

    async function submitAddFavorite() {
        const text = favoriteInput.trim();
        if (!text || addFavoriteSaving) return;

        addFavoriteSaving = true;
        try {
            await invoke('add_custom_favorite_record', { content: text });
            addFavoriteOpen = false;
            favoriteInput = '';
            await loadHistory();
        } catch (error) {
            console.error('Failed to add custom favorite record:', error);
        } finally {
            addFavoriteSaving = false;
        }
    }

    async function handleClearAll() {
        clearConfirmOpen = true;
    }

    async function confirmClearAll() {
        try {
            await invoke('clear_clipboard_history');
            records = [];
        } catch (error) {
            console.error('Failed to clear history:', error);
        } finally {
            clearConfirmOpen = false;
        }
    }

    async function toggleFavoritesView() {
        favoritesOnly = !favoritesOnly;
        searchKeyword = '';
        await loadHistory(false);
    }

    function openExportFromSettings() {
        settingsOpen = false;
        void (async () => {
            try {
                await invoke('suspend_auto_hide', { ms: 10000 });
                const selected = await openDialog({
                    multiple: false,
                    directory: true
                });
                if (!selected || Array.isArray(selected)) return;

                const result = await invoke<ExportFavoritesResult>('export_favorites_to_path', {
                    path: selected,
                });
                window.alert(`导出完成，共 ${result.count} 条收藏\n文件: ${result.path}`);
            } catch (error) {
                window.alert(`导出失败: ${String(error)}`);
            }
        })();
    }

    function openImportFromSettings() {
        settingsOpen = false;
        void (async () => {
            try {
                await invoke('suspend_auto_hide', { ms: 10000 });
                const selected = await openDialog({
                    multiple: false,
                    directory: false,
                    filters: [{ name: 'JSON', extensions: ['json'] }]
                });
                if (!selected || Array.isArray(selected)) return;
                const count = await invoke<number>('import_favorites_from_path', { path: selected });
                window.alert(`导入完成，新增 ${count} 条收藏`);
                await loadHistory();
            } catch (error) {
                window.alert(`导入失败: ${String(error)}`);
            }
        })();
    }

    onMount(() => {
        // 监听由后端启动，这里仅负责刷新 UI
        refreshInterval = setInterval(refreshHistory, 900);
        loadSettings();
        getVersion()
            .then((v) => {
                appVersion = v;
            })
            .catch(() => {});
        listen('open-settings', async () => {
            await loadSettings();
            settingsOpen = true;
        }).then((unlisten) => {
            unlistenOpenSettings = unlisten;
        }).catch((error) => {
            console.error('Failed to listen open-settings:', error);
        });
        listen('open-about', () => {
            settingsOpen = false;
            aboutOpen = true;
        }).then((unlisten) => {
            unlistenOpenAbout = unlisten;
        }).catch((error) => {
            console.error('Failed to listen open-about:', error);
        });
        listen('main-window-opened', () => {
            void resetSearchStateOnShow();
        }).then((unlisten) => {
            unlistenMainWindowOpened = unlisten;
        }).catch((error) => {
            console.error('Failed to listen main-window-opened:', error);
        });

        void refreshHistory();

        return () => {
            if (unlistenOpenSettings) {
                unlistenOpenSettings();
            }
            if (unlistenOpenAbout) {
                unlistenOpenAbout();
            }
            if (unlistenMainWindowOpened) {
                unlistenMainWindowOpened();
            }
            if (refreshInterval) {
                clearInterval(refreshInterval);
            }
        };
    });
</script>

<main class="app">
    <header class="header">
        <h1>{pageTitle()}</h1>
        <div class="header-actions">
            {#if !favoritesOnly}
                <button class="refresh-btn danger" onclick={handleClearAll} aria-label="清空历史">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
                        <path d="M10 11v6M14 11v6"/>
                    </svg>
                </button>
            {/if}
            <button
                class="refresh-btn favorite-toggle"
                class:active={favoritesOnly}
                onclick={toggleFavoritesView}
                aria-label={favoritesOnly ? '切换到历史记录' : '切换到收藏'}
            >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 3l2.9 5.88 6.49.95-4.7 4.58 1.11 6.47L12 17.8l-5.8 3.08 1.1-6.47-4.7-4.58 6.5-.95z"/>
                </svg>
            </button>
            <button class="refresh-btn" onclick={openAddFavoriteDialog} aria-label="添加收藏">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M12 5v14M5 12h14"/>
                </svg>
            </button>
        </div>
    </header>

    <div class="search-container">
        <SearchBar
            bind:value={searchKeyword}
            onchange={handleSearch}
        />
    </div>

    <div class="list-container">
        <ClipboardList
            {records}
            {loading}
            oncopy={handleCopy}
            ondelete={handleDelete}
            onfavorite={handleFavorite}
            onpin={handlePinned}
            emptyTitle={emptyTitle()}
            emptyHint={emptyHint()}
        />
    </div>

    {#if addFavoriteOpen}
        <div class="confirm-backdrop">
            <div class="confirm-modal" role="dialog" aria-modal="true" aria-label="添加收藏">
                <h3>添加收藏文本</h3>
                <p>输入你常用的文本，保存后会直接加入收藏。</p>
                <textarea
                    class="favorite-input"
                    bind:value={favoriteInput}
                    rows="4"
                    placeholder="输入收藏内容..."
                ></textarea>
                <div class="confirm-actions">
                    <button class="cancel-btn" onclick={closeAddFavoriteDialog} disabled={addFavoriteSaving}>取消</button>
                    <button class="primary-btn" onclick={submitAddFavorite} disabled={addFavoriteSaving || !favoriteInput.trim()}>
                        {addFavoriteSaving ? '保存中...' : '添加'}
                    </button>
                </div>
            </div>
        </div>
    {/if}

    <SettingsModal
        open={settingsOpen}
        {settings}
        onsave={saveSettings}
        onopenimport={openImportFromSettings}
        onopenexport={openExportFromSettings}
        onclose={() => (settingsOpen = false)}
    />

    {#if aboutOpen}
        <div class="confirm-backdrop">
            <div class="confirm-modal" role="dialog" aria-modal="true" aria-label="关于">
                <h3>关于 Clipper</h3>
                <p>版本：v{appVersion}</p>
                <p>作者：Jiaxin</p>
                <div class="confirm-actions">
                    <button class="primary-btn" onclick={() => (aboutOpen = false)}>知道了</button>
                </div>
            </div>
        </div>
    {/if}

    {#if clearConfirmOpen}
        <div class="confirm-backdrop">
            <div class="confirm-modal" role="alertdialog" aria-modal="true" aria-label="确认清空">
                <h3>确认清空历史</h3>
                <p>将删除全部历史记录，且无法撤销。</p>
                <div class="confirm-actions">
                    <button class="cancel-btn" onclick={() => (clearConfirmOpen = false)}>取消</button>
                    <button class="danger-btn" onclick={confirmClearAll}>清空</button>
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        overflow: hidden;
    }

    :global(*) {
        box-sizing: border-box;
    }

    :global(:root) {
        --bg-primary: #ffffff;
        --bg-secondary: #f3f4f6;
        --bg-hover: #f9fafb;
        --text-primary: #111827;
        --text-secondary: #6b7280;
        --text-tertiary: #9ca3af;
        --border-color: #e5e7eb;
        --accent-color: #2563eb;
        --accent-light: #eff6ff;
        --danger-color: #ef4444;
        --danger-light: #fef2f2;
        --scrollbar-track: #eef2f7;
        --scrollbar-thumb: #c6cdd8;
        --scrollbar-thumb-hover: #aeb7c4;
    }

    :global([data-theme="dark"]) {
        --bg-primary: #1f2937;
        --bg-secondary: #374151;
        --bg-hover: #4b5563;
        --text-primary: #f9fafb;
        --text-secondary: #9ca3af;
        --text-tertiary: #6b7280;
        --border-color: #4b5563;
        --accent-color: #60a5fa;
        --accent-light: #1e3a5f;
        --danger-color: #f87171;
        --danger-light: #7f1d1d;
        --scrollbar-track: #303745;
        --scrollbar-thumb: #596275;
        --scrollbar-thumb-hover: #727e95;
    }

    :global(*) {
        scrollbar-width: thin;
        scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
    }

    :global(*::-webkit-scrollbar) {
        width: 10px;
        height: 10px;
    }

    :global(*::-webkit-scrollbar-track) {
        background: var(--scrollbar-track);
    }

    :global(*::-webkit-scrollbar-thumb) {
        background: var(--scrollbar-thumb);
        border-radius: 8px;
        border: 2px solid var(--scrollbar-track);
    }

    :global(*::-webkit-scrollbar-thumb:hover) {
        background: var(--scrollbar-thumb-hover);
    }

    .app {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: var(--bg-primary);
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    }

    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px;
        border-bottom: 1px solid var(--border-color);
        background: var(--bg-primary);
    }

    h1 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .header-actions {
        display: flex;
        gap: 8px;
    }

    .refresh-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        padding: 0;
        border: none;
        background: transparent;
        cursor: pointer;
        border-radius: 6px;
        transition: background-color 0.15s;
    }

    .refresh-btn:hover {
        background: var(--bg-hover);
    }

    .refresh-btn.danger:hover {
        background: var(--danger-light);
    }

    .refresh-btn.danger:hover svg {
        color: var(--danger-color);
    }

    .refresh-btn svg {
        width: 18px;
        height: 18px;
        color: var(--text-secondary);
    }

    .search-container {
        padding: 12px 16px;
        border-bottom: 1px solid var(--border-color);
    }

    .list-container {
        flex: 1;
        min-height: 0;
        display: flex;
        overflow: hidden;
    }

    .favorite-toggle.active {
        background: rgba(245, 158, 11, 0.14);
    }

    .favorite-toggle.active svg {
        color: #f59e0b;
        fill: rgba(245, 158, 11, 0.22);
    }

    .favorite-input {
        width: 100%;
        min-height: 92px;
        resize: vertical;
        margin-top: 10px;
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 8px 10px;
        font-size: 13px;
        color: var(--text-primary);
        background: var(--bg-primary);
        outline: none;
    }

    .favorite-input:focus {
        border-color: var(--accent-color);
    }

    .confirm-backdrop {
        position: fixed;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 16px;
        background: rgba(17, 24, 39, 0.5);
        z-index: 50;
    }

    .confirm-modal {
        width: min(92vw, 360px);
        max-width: 100%;
        background: var(--bg-primary);
        border: 1px solid var(--border-color);
        border-radius: 10px;
        padding: 14px;
        box-shadow: 0 16px 40px rgba(0, 0, 0, 0.2);
    }

    .confirm-modal h3 {
        margin: 0 0 8px 0;
        font-size: 15px;
        color: var(--text-primary);
    }

    .confirm-modal p {
        margin: 0;
        font-size: 13px;
        color: var(--text-secondary);
        line-height: 1.4;
    }

    .confirm-actions {
        margin-top: 14px;
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }

    .cancel-btn,
    .danger-btn,
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

    .danger-btn {
        border-color: var(--danger-color);
        background: var(--danger-color);
        color: #fff;
    }

    .primary-btn {
        border-color: var(--accent-color);
        background: var(--accent-color);
        color: #fff;
    }
</style>
