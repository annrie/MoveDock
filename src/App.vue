<script setup lang="ts">
import { onMounted } from 'vue'
import { useMoveDock } from './composables/useMoveDock'
import { targets, statusLabel } from './types'
import AppIcon from './components/AppIcon.vue'
import LogPanel from './components/LogPanel.vue'
const { native, data, settingsDirty, site, view, inspection, environment, direction, selectedTargets, document, dirty, busy, running, stopping, error, notice, logs, diagnostics, lastResult, locked, remote, ready, initialize, selectSite, addSite, loadEnvironments, editMovefile, reloadDocument, saveDocument, removeSite, saveSettings, autofillSettings, checkTools, chooseGemfile, run, stop } = useMoveDock()
onMounted(initialize)
const date = (value: string) => new Date(value).toLocaleString('ja-JP', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand"><span class="brand-mark"><AppIcon name="dock" :size="26" /></span><div><strong>MoveDock</strong><span>WORDPRESS SYNC</span></div></div>
      <div class="sidebar-section"><span class="eyebrow">ワークスペース</span><span class="count">{{ data.sites.length }}</span></div>
      <nav aria-label="登録サイト" class="site-list">
        <button v-for="entry in data.sites" :key="entry.id" class="site-item" :class="{ active: site?.id === entry.id && (view === 'sync' || view === 'editor') }" :disabled="locked" @click="selectSite(entry)"><span class="site-monogram">{{ entry.name.slice(0, 1).toUpperCase() }}</span><span class="site-name">{{ entry.name }}<small>WordPress</small></span><span v-if="site?.id === entry.id" class="selection-dot" /></button>
        <div v-if="!data.sites.length" class="sidebar-empty">まだサイトがありません</div>
      </nav>
      <button class="add-site" :disabled="!native || locked" @click="addSite()"><AppIcon name="plus" :size="17" /> サイトを追加</button>
      <div class="sidebar-bottom"><button class="nav-item" :class="{ active: view === 'history' }" @click="view = 'history'"><AppIcon name="clock" :size="18" /> 実行履歴</button><button class="nav-item" :class="{ active: view === 'settings' }" @click="view = 'settings'"><AppIcon name="settings" :size="18" /> 設定</button><div class="sidebar-note"><AppIcon name="shield" :size="16" /><span>Powered by Wordmove<br /><small>すべての操作は、この Mac から。</small></span></div></div>
    </aside>
    <main class="main">
      <header class="topbar"><div class="breadcrumb">MoveDock <span>/</span> {{ view === 'settings' ? '設定' : view === 'history' ? '実行履歴' : site?.name || 'ワークスペース' }}</div><span class="platform"><span class="status-dot" /> macOS Universal</span></header>
      <div class="content-scroll" :class="{ 'sync-page': view === 'sync' && site }">
        <div v-if="!native" class="banner info">ブラウザプレビューです。ファイル操作・同期は Tauri アプリで利用できます。</div>
        <div v-if="error" class="banner error" role="alert"><strong>操作を完了できませんでした</strong><pre>{{ error }}</pre><button class="text-button" @click="error = ''">閉じる</button></div>
        <div v-if="notice" class="banner success" role="status">{{ notice }}</div>

        <template v-if="view === 'sync' || view === 'editor'">
          <template v-if="!site">
            <section class="welcome">
              <span class="eyebrow accent">A PLACE FOR EVERY MOVE</span><h1>サイトの移動に、<br />確かな手応えを。</h1><p>ローカルからリモートへ。リモートからローカルへ。<br />WordPress の同期を、ひとつのワークスペースで。</p>
              <div class="connection-illustration" aria-hidden="true"><div class="illustration-node"><AppIcon name="laptop" :size="32" /><span>LOCAL</span></div><div class="illustration-route"><span /><AppIcon name="dock" :size="28" /><span /></div><div class="illustration-node remote-node"><AppIcon name="server" :size="32" /><span>REMOTE</span></div></div>
              <div class="welcome-actions"><button class="primary" :disabled="!native || locked" @click="addSite()"><AppIcon name="plus" :size="18" /> Movefile を開く</button><button class="secondary" :disabled="!native || locked" @click="addSite(true)">新しく作成する</button></div>
              <p class="small-note">既存の Movefile をそのまま利用できます。</p>
            </section>
            <div class="onboarding-steps"><div><span>01</span><h2>サイトを登録</h2><p>Movefile を選んで<br />ワークスペースに追加。</p></div><div><span>02</span><h2>環境と対象を選択</h2><p>テーマ、メディア、DB。<br />必要なものだけを同期。</p></div><div><span>03</span><h2>確認して、実行</h2><p>シミュレーションで確認し、<br />ログを見ながら進行。</p></div></div>
          </template>
          <template v-else>
            <div class="page-heading"><div><span class="eyebrow accent">SITE WORKSPACE</span><h1>{{ site.name }}</h1><p class="file-path" :title="site.path"><AppIcon name="folder" :size="14" />{{ site.path }}</p></div><button class="secondary small" :disabled="locked || dirty" @click="loadEnvironments"><AppIcon name="refresh" :size="15" />{{ inspection ? '環境を再読み込み' : '環境を読み込む' }}</button></div>
            <nav class="tabs" aria-label="サイトの操作"><button :aria-current="view === 'sync' ? 'page' : undefined" :class="{ selected: view === 'sync' }" @click="view = 'sync'"><AppIcon name="dock" :size="17" />同期</button><button :aria-current="view === 'editor' ? 'page' : undefined" :class="{ selected: view === 'editor' }" :disabled="locked" @click="editMovefile"><AppIcon name="file" :size="17" />Movefile <span v-if="dirty" class="unsaved-dot" /></button></nav>
            <template v-if="view === 'sync'">
              <div v-if="!inspection" class="load-hint"><AppIcon name="shield" :size="19" /><div><strong>Movefile を確認して、環境を読み込みます</strong><p>ERB が含まれる場合は Ruby コードが実行されます。信頼できる設定を使用してください。</p></div></div>
              <div class="sync-route"><div class="endpoint"><div class="endpoint-label"><AppIcon name="laptop" :size="18" /><span>LOCAL</span><span class="endpoint-tag">この Mac</span></div><strong>{{ inspection?.local || 'ローカルサイト' }}</strong><span>{{ direction === 'push' ? '同期元' : '同期先 · 上書きされる側' }}</span></div><div class="route-arrow" :class="{ reverse: direction === 'pull' }"><AppIcon name="arrow" :size="23" /></div><div class="endpoint remote-endpoint"><div class="endpoint-label"><AppIcon name="server" :size="18" /><label for="environment">REMOTE</label><select id="environment" v-model="environment" :disabled="!inspection || locked"><option v-if="!inspection" value="">未読み込み</option><option v-for="env in inspection?.environments" :key="env.name" :value="env.name">{{ env.name }}</option></select></div><strong>{{ remote?.vhost || 'リモートサイト' }}</strong><span>{{ direction === 'pull' ? '同期元' : '同期先 · 上書きされる側' }}</span></div></div>
              <section class="sync-options"><div class="section-heading"><div><h2>同期する内容</h2><p>必要な対象だけを選択してください。</p></div><fieldset class="direction-switch" :disabled="locked"><legend class="sr-only">同期方向</legend><label :class="{ chosen: direction === 'pull' }"><input v-model="direction" type="radio" value="pull" /> <AppIcon name="arrow" class="rotate" :size="14" />Pull</label><label :class="{ chosen: direction === 'push' }"><input v-model="direction" type="radio" value="push" />Push<AppIcon name="arrow" :size="14" /></label></fieldset></div>
                <fieldset class="target-grid" :disabled="locked"><legend class="sr-only">同期対象</legend><label v-for="target in targets" :key="target.id" class="target" :class="{ checked: selectedTargets.includes(target.id), database: target.id === 'db' }"><input v-model="selectedTargets" type="checkbox" :value="target.id" /><span><strong>{{ target.label }}</strong><small>{{ target.detail }}</small></span><span v-if="target.id === 'db'" class="db-label">上書き対象</span></label></fieldset>
                <div class="run-actions"><div class="run-note"><AppIcon name="shield" :size="17" /><span v-if="settingsDirty">設定の変更を保存してください。</span><span v-else-if="dirty">Movefile の変更を保存してください。</span><span v-else-if="!inspection">環境を読み込むと実行できます。</span><span v-else-if="!selectedTargets.length">同期対象を選択してください。</span><span v-else>まずはシミュレーションで確認。</span></div><div class="row"><button class="secondary" :disabled="!ready" @click="run(true)">シミュレーション</button><button class="primary" :disabled="!ready" @click="run(false)"><AppIcon name="play" :size="15" />{{ direction === 'pull' ? 'Pull' : 'Push' }} を実行</button></div></div>
              </section>
              <p class="operation-note">シミュレーションでも ERB の評価・接続は発生します。停止しても変更は元に戻りません。</p>
              <div v-if="lastResult" class="result-line" role="status">{{ statusLabel[lastResult.status] }} · {{ lastResult.request.simulate ? 'シミュレーション' : '同期' }} · {{ date(lastResult.finishedAt) }} <span v-if="lastResult.status === 'success'">（CLI の終了結果です。スキップされた対象はログを確認してください。）</span></div>
            </template>
            <section v-else class="editor-section"><div class="section-heading"><div><h2>Movefile エディター</h2><p>コメント・ERB・YAML アンカーを保持して編集できます。</p></div><span class="badge">{{ dirty ? '未保存' : '保存済み' }}</span></div><label for="movefile-content" class="sr-only">Movefile の内容</label><textarea v-if="document" id="movefile-content" v-model="document.content" class="code-editor" spellcheck="false" :disabled="locked" /><div class="editor-actions"><button class="text-button danger" :disabled="locked" @click="removeSite">サイト登録を解除</button><div class="row"><button class="secondary" :disabled="locked" @click="reloadDocument">再読み込み</button><button class="primary" :disabled="locked || !dirty" @click="saveDocument">変更を保存</button></div></div></section>
          </template>
        </template>

        <template v-else-if="view === 'settings'">
          <div class="page-heading"><div><span class="eyebrow accent">PREFERENCES</span><h1>設定</h1><p>Wordmove を実行する環境を整えます。</p></div></div>
          <section class="setup-summary" aria-labelledby="setup-heading">
            <div class="section-heading"><div><h2 id="setup-heading">実行環境の自動設定</h2><p>初回起動時に検出した項目を入力・保存しています。必要な場合だけ変更してください。</p></div><button class="secondary" :disabled="locked || !native || settingsDirty" @click="autofillSettings">空欄を再検出</button></div>
            <ul v-if="data.setupNotes?.length" class="setup-notes"><li v-for="note in data.setupNotes" :key="note">{{ note }}</li></ul>
            <p v-if="settingsDirty" class="setup-notes" role="status">再検出・動作確認の前に、変更した設定を保存してください。</p>
          </section>
          <form class="settings-form" @submit.prevent="saveSettings"><fieldset :disabled="locked || !native"><legend>実行環境</legend><p class="section-description">Wordmove フォークが見つかった場合は Bundler を設定します。見つからない場合は導入済みの Wordmove を検出します。</p><div class="form-grid"><label>実行方式<select v-model="data.settings.mode"><option value="bundler">Bundler（bundle exec wordmove）</option><option value="direct">Wordmove を直接実行</option></select></label><label>実行ファイル<input v-model="data.settings.executable" :placeholder="data.settings.mode === 'bundler' ? '自動検出: bundle' : '自動検出: wordmove'" spellcheck="false" /></label></div><label v-if="data.settings.mode === 'bundler'">Wordmove の Gemfile<div class="input-with-button"><input v-model="data.settings.gemfile" placeholder="/path/to/wordmove/Gemfile" spellcheck="false" /><button type="button" class="secondary" @click="chooseGemfile">選択</button></div></label><div class="form-grid"><label>追加 PATH <span class="label-hint">1 行に 1 ディレクトリ</span><textarea v-model="data.settings.extraPath" rows="3" placeholder="追加不要の場合は空欄" spellcheck="false" /></label><label>Ruby バージョン <span class="label-hint">rbenv を使う場合</span><input v-model="data.settings.rubyVersion" placeholder="例: 3.3.12" spellcheck="false" /><small>Homebrew / Local の PHP・MySQL も必要に応じて自動検出します。</small></label></div></fieldset><fieldset :disabled="locked"><legend>外観</legend><label class="theme-field">カラーテーマ<select v-model="data.settings.theme"><option value="system">システムに合わせる</option><option value="light">ライト</option><option value="dark">ダーク</option></select></label></fieldset><div class="settings-actions"><span>変更後に設定を保存してください。</span><button class="primary" :disabled="locked || !native">設定を保存</button></div></form>
          <section class="diagnostics"><div class="section-heading"><div><h2>依存コマンド</h2><p>保存済みの設定で確認します。FTP 利用時のみ lftp が必要です。</p></div><button class="secondary" :disabled="locked || !native || settingsDirty" @click="checkTools">実行環境を確認</button></div><div v-for="tool in diagnostics" :key="tool.name" class="diagnostic"><span class="tool-status" :class="{ available: tool.available }"><AppIcon v-if="tool.available" name="check" :size="13" /><span v-else>!</span></span><strong>{{ tool.name }}</strong><pre>{{ tool.detail }}</pre></div><p v-if="!diagnostics.length" class="empty-note">Wordmove、Ruby、SSH、rsync、WP-CLI などを確認できます。</p></section>
        </template>
        <template v-else>
          <div class="page-heading"><div><span class="eyebrow accent">ACTIVITY</span><h1>実行履歴</h1><p>この Mac で実行した、直近 200 件の記録。</p></div><span class="badge">{{ data.history.length }} 件</span></div>
          <div v-if="!data.history.length" class="history-empty"><AppIcon name="clock" :size="38" /><h2>最初の同期を待っています</h2><p>シミュレーションや同期の結果がここに記録されます。</p></div>
          <div v-else class="history-table-wrap"><table><thead><tr><th>サイト / 日時</th><th>操作</th><th>環境 / 対象</th><th>結果</th></tr></thead><tbody><tr v-for="entry in data.history" :key="entry.id"><td><strong>{{ entry.siteName }}</strong><small>{{ date(entry.startedAt) }}</small></td><td><span class="history-direction">{{ entry.request.direction.toUpperCase() }}</span><small>{{ entry.request.simulate ? 'シミュレーション' : '同期' }}</small></td><td>{{ entry.request.environment }}<small>{{ entry.request.targets.join(', ') }}</small></td><td><span class="result-badge" :class="entry.status">{{ statusLabel[entry.status] }}</span><small>exit {{ entry.exitCode ?? '—' }}</small></td></tr></tbody></table></div>
          <p class="operation-note">ログ本文・接続パスワードは履歴に保存しません。</p>
        </template>
      </div>
      <LogPanel v-if="view === 'sync' && site || running" :logs="logs" :running="running" :stopping="stopping" @stop="stop" />
      <footer class="statusbar"><span role="status"><span class="status-dot" :class="{ working: locked }" />{{ running ? (stopping ? '停止処理中。完了までお待ちください。' : 'Wordmove を実行中… 終了するには先に停止してください。') : busy || '準備完了' }}</span><span>MoveDock <span class="version">0.1.0</span></span></footer>
    </main>
  </div>
</template>
