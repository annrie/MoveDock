<script setup lang="ts">
import { onMounted } from 'vue'
import { useMoveDock } from './composables/useMoveDock'
import { targets } from './types'
import { t, language, supportedLocales, formatDate as date, statusText } from './i18n'
import { formatAppMessage, formatSetupNote, errorHint } from './i18n/appMessages'
import AppIcon from './components/AppIcon.vue'
import LogPanel from './components/LogPanel.vue'
const { native, data, settingsDirty, site, view, inspection, environment, direction, selectedTargets, document, dirty, busy, running, stopping, error, notice, logs, diagnostics, lastResult, runError, locked, remote, ready, initialize, selectSite, addSite, loadEnvironments, editMovefile, reloadDocument, saveDocument, removeSite, saveSettings, autofillSettings, checkTools, chooseGemfile, run, stop } = useMoveDock()
onMounted(initialize)
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand"><span class="brand-mark"><AppIcon name="dock" :size="26" /></span><div><strong>MoveDock</strong><span>WORDPRESS SYNC</span></div></div>
      <div class="sidebar-section"><span class="eyebrow">{{ t('nav.workspace') }}</span><span class="count">{{ data.sites.length }}</span></div>
      <nav :aria-label="t('nav.sites')" class="site-list">
        <button v-for="entry in data.sites" :key="entry.id" class="site-item" :class="{ active: site?.id === entry.id && (view === 'sync' || view === 'editor') }" :disabled="locked" @click="selectSite(entry)"><span class="site-monogram">{{ entry.name.slice(0, 1).toUpperCase() }}</span><span class="site-name">{{ entry.name }}<small>WordPress</small></span><span v-if="site?.id === entry.id" class="selection-dot" /></button>
        <div v-if="!data.sites.length" class="sidebar-empty">{{ t('nav.empty') }}</div>
      </nav>
      <button class="add-site" :disabled="!native || locked" @click="addSite()"><AppIcon name="plus" :size="17" /> {{ t('nav.add') }}</button>
      <div class="sidebar-bottom"><button class="nav-item" :class="{ active: view === 'history' }" @click="view = 'history'"><AppIcon name="clock" :size="18" /> {{ t('nav.history') }}</button><button class="nav-item" :class="{ active: view === 'settings' }" @click="view = 'settings'"><AppIcon name="settings" :size="18" /> {{ t('nav.settings') }}</button><div class="sidebar-note"><AppIcon name="shield" :size="16" /><span>Powered by Wordmove<br /><small>{{ t('nav.footer') }}</small></span></div></div>
    </aside>
    <main class="main">
      <header class="topbar"><div class="breadcrumb">MoveDock <span>/</span> {{ view === 'settings' ? t('nav.settings') : view === 'history' ? t('nav.history') : site?.name || t('nav.workspace') }}</div><span class="platform"><span class="status-dot" /> macOS Universal</span></header>
      <div class="content-scroll" :class="{ 'sync-page': view === 'sync' && site }">
        <div v-if="!native" class="banner info">{{ t('preview.notice') }}</div>
        <div v-if="error" class="banner error" role="alert"><strong>{{ t('error.title') }}</strong><p v-if="errorHint(error)">{{ errorHint(error) }}</p><pre>{{ formatAppMessage(error) }}</pre><button class="text-button" @click="error = ''">{{ t('common.close') }}</button></div>
        <div v-if="notice" class="banner success" role="status">{{ t(notice) }}</div>

        <template v-if="view === 'sync' || view === 'editor'">
          <template v-if="!site">
            <section class="welcome">
              <span class="eyebrow accent">{{ t('welcome.eyebrow') }}</span><h1>{{ t('welcome.title') }}</h1><p>{{ t('welcome.description') }}</p>
              <div class="connection-illustration" aria-hidden="true"><div class="illustration-node"><AppIcon name="laptop" :size="32" /><span>{{ t('common.local') }}</span></div><div class="illustration-route"><span /><AppIcon name="dock" :size="28" /><span /></div><div class="illustration-node remote-node"><AppIcon name="server" :size="32" /><span>{{ t('common.remote') }}</span></div></div>
              <div class="welcome-actions"><button class="primary" :disabled="!native || locked" @click="addSite()"><AppIcon name="plus" :size="18" /> {{ t('welcome.open') }}</button><button class="secondary" :disabled="!native || locked" @click="addSite(true)">{{ t('welcome.create') }}</button></div>
              <p class="small-note">{{ t('welcome.note') }}</p>
            </section>
            <div class="onboarding-steps"><div><span>01</span><h2>{{ t('welcome.step1') }}</h2><p>{{ t('welcome.detail1') }}</p></div><div><span>02</span><h2>{{ t('welcome.step2') }}</h2><p>{{ t('welcome.detail2') }}</p></div><div><span>03</span><h2>{{ t('welcome.step3') }}</h2><p>{{ t('welcome.detail3') }}</p></div></div>
          </template>
          <template v-else>
            <div class="page-heading"><div><span class="eyebrow accent">{{ t('site.eyebrow') }}</span><h1>{{ site.name }}</h1><p class="file-path" :title="site.path"><AppIcon name="folder" :size="14" />{{ site.path }}</p></div><button class="secondary small" :disabled="locked || dirty" @click="loadEnvironments"><AppIcon name="refresh" :size="15" />{{ inspection ? t('site.reload') : t('site.load') }}</button></div>
            <nav class="tabs" :aria-label="t('site.operations')"><button :aria-current="view === 'sync' ? 'page' : undefined" :class="{ selected: view === 'sync' }" @click="view = 'sync'"><AppIcon name="dock" :size="17" />{{ t('common.sync') }}</button><button :aria-current="view === 'editor' ? 'page' : undefined" :class="{ selected: view === 'editor' }" :disabled="locked" @click="editMovefile"><AppIcon name="file" :size="17" />Movefile <span v-if="dirty" class="unsaved-dot" /></button></nav>
            <template v-if="view === 'sync'">
              <div v-if="!inspection" class="load-hint"><AppIcon name="shield" :size="19" /><div><strong>{{ t('site.trustTitle') }}</strong><p>{{ t('site.trustDescription') }}</p></div></div>
              <div class="sync-route"><div class="endpoint"><div class="endpoint-label"><AppIcon name="laptop" :size="18" /><span>{{ t('common.local') }}</span><span class="endpoint-tag">{{ t('site.thisMac') }}</span></div><strong>{{ inspection?.local || t('site.local') }}</strong><span>{{ direction === 'push' ? t('site.source') : t('site.destination') }}</span></div><div class="route-arrow" :class="{ reverse: direction === 'pull' }"><AppIcon name="arrow" :size="23" /></div><div class="endpoint remote-endpoint"><div class="endpoint-label"><AppIcon name="server" :size="18" /><label for="environment">{{ t('common.remote') }}</label><select id="environment" v-model="environment" :disabled="!inspection || locked"><option v-if="!inspection" value="">{{ t('site.unloaded') }}</option><option v-for="env in inspection?.environments" :key="env.name" :value="env.name">{{ env.name }}</option></select></div><strong>{{ remote?.vhost || t('site.remote') }}</strong><span>{{ direction === 'pull' ? t('site.source') : t('site.destination') }}</span></div></div>
              <section class="sync-options"><div class="section-heading"><div><h2>{{ t('sync.title') }}</h2><p>{{ t('sync.description') }}</p></div><fieldset class="direction-switch" :disabled="locked"><legend class="sr-only">{{ t('sync.direction') }}</legend><label :class="{ chosen: direction === 'pull' }"><input v-model="direction" type="radio" value="pull" /> <AppIcon name="arrow" class="rotate" :size="14" />Pull</label><label :class="{ chosen: direction === 'push' }"><input v-model="direction" type="radio" value="push" />Push<AppIcon name="arrow" :size="14" /></label></fieldset></div>
                <fieldset class="target-grid" :disabled="locked"><legend class="sr-only">{{ t('sync.targets') }}</legend><label v-for="target in targets" :key="target.id" class="target" :class="{ checked: selectedTargets.includes(target.id), database: target.id === 'db' }"><input v-model="selectedTargets" type="checkbox" :value="target.id" /><span><strong>{{ t(`target.${target.id}`) }}</strong><small>{{ target.detail }}</small></span><span v-if="target.id === 'db'" class="db-label">{{ t('sync.overwrite') }}</span></label></fieldset>
                <div class="run-actions"><div class="run-note"><AppIcon name="shield" :size="17" /><span v-if="settingsDirty">{{ t('sync.saveSettings') }}</span><span v-else-if="dirty">{{ t('sync.saveMovefile') }}</span><span v-else-if="!inspection">{{ t('sync.loadFirst') }}</span><span v-else-if="!selectedTargets.length">{{ t('sync.selectTargets') }}</span><span v-else>{{ t('sync.trySimulation') }}</span></div><div class="row"><button class="secondary" :disabled="!ready" @click="run(true)">{{ t('common.simulation') }}</button><button class="primary" :disabled="!ready" @click="run(false)"><AppIcon name="play" :size="15" />{{ t('common.execute', { direction: direction === 'pull' ? 'Pull' : 'Push' }) }}</button></div></div>
              </section>
              <p class="operation-note">{{ t('sync.warning') }}</p>
              <div v-if="lastResult" class="result-line" role="status">{{ statusText(lastResult.status) }} · {{ lastResult.request.simulate ? t('common.simulation') : t('common.sync') }} · {{ date(lastResult.finishedAt) }} <span v-if="lastResult.status === 'success'">{{ t('sync.resultNote') }}</span></div>
            </template>
            <section v-else class="editor-section"><div class="section-heading"><div><h2>{{ t('editor.title') }}</h2><p>{{ t('editor.description') }}</p></div><span class="badge">{{ dirty ? t('editor.unsaved') : t('editor.saved') }}</span></div><label for="movefile-content" class="sr-only">{{ t('editor.content') }}</label><textarea v-if="document" id="movefile-content" v-model="document.content" class="code-editor" spellcheck="false" :disabled="locked" /><div class="editor-actions"><button class="text-button danger" :disabled="locked" @click="removeSite">{{ t('editor.remove') }}</button><div class="row"><button class="secondary" :disabled="locked" @click="reloadDocument">{{ t('common.reload') }}</button><button class="primary" :disabled="locked || !dirty" @click="saveDocument">{{ t('editor.save') }}</button></div></div></section>
          </template>
        </template>

        <template v-else-if="view === 'settings'">
          <div class="page-heading"><div><span class="eyebrow accent">{{ t('settings.eyebrow') }}</span><h1>{{ t('nav.settings') }}</h1><p>{{ t('settings.description') }}</p></div></div>
          <section class="setup-summary" aria-labelledby="setup-heading">
            <div class="section-heading"><div><h2 id="setup-heading">{{ t('settings.auto') }}</h2><p>{{ t('settings.autoDescription') }}</p></div><button class="secondary" :disabled="locked || !native || settingsDirty" @click="autofillSettings">{{ t('settings.detect') }}</button></div>
            <ul v-if="data.setupNotes?.length" class="setup-notes"><li v-for="note in data.setupNotes" :key="note">{{ formatSetupNote(note) }}</li></ul>
            <p v-if="settingsDirty" class="setup-notes" role="status">{{ t('settings.saveFirst') }}</p>
          </section>
          <section class="language-section" aria-labelledby="language-heading"><label id="language-heading" for="display-language">{{ t('settings.language') }}</label><select id="display-language" v-model="language"><option v-for="item in supportedLocales" :key="item.value" :value="item.value" :lang="item.value">{{ item.label }}</option></select><p>{{ t('settings.languageHint') }}</p></section>
          <form class="settings-form" @submit.prevent="saveSettings"><fieldset :disabled="locked || !native"><legend>{{ t('settings.runtime') }}</legend><p class="section-description">{{ t('settings.runtimeDescription') }}</p><div class="form-grid"><label>{{ t('settings.mode') }}<select v-model="data.settings.mode"><option value="bundler">{{ t('settings.bundler') }}</option><option value="direct">{{ t('settings.direct') }}</option></select></label><label>{{ t('settings.executable') }}<input v-model="data.settings.executable" :placeholder="t('settings.autoPath', { name: data.settings.mode === 'bundler' ? 'bundle' : 'wordmove' })" spellcheck="false" /></label></div><label v-if="data.settings.mode === 'bundler'">{{ t('settings.gemfile') }}<div class="input-with-button"><input v-model="data.settings.gemfile" placeholder="/path/to/wordmove/Gemfile" spellcheck="false" /><button type="button" class="secondary" @click="chooseGemfile">{{ t('settings.choose') }}</button></div></label><div class="form-grid"><label>{{ t('settings.path') }} <span class="label-hint">{{ t('settings.pathHint') }}</span><textarea v-model="data.settings.extraPath" rows="3" :placeholder="t('settings.pathPlaceholder')" spellcheck="false" /></label><label>{{ t('settings.ruby') }} <span class="label-hint">{{ t('settings.rubyHint') }}</span><input v-model="data.settings.rubyVersion" :placeholder="t('settings.rubyExample')" spellcheck="false" /><small>{{ t('settings.toolsHint') }}</small></label></div></fieldset><fieldset :disabled="locked"><legend>{{ t('settings.appearance') }}</legend><label class="theme-field">{{ t('settings.theme') }}<select v-model="data.settings.theme"><option value="system">{{ t('settings.system') }}</option><option value="light">{{ t('settings.light') }}</option><option value="dark">{{ t('settings.dark') }}</option></select></label></fieldset><div class="settings-actions"><span>{{ t('settings.saveHint') }}</span><button class="primary" :disabled="locked || !native">{{ t('settings.save') }}</button></div></form>
          <section class="diagnostics"><div class="section-heading"><div><h2>{{ t('settings.dependencies') }}</h2><p>{{ t('settings.diagnosticsDescription') }}</p></div><button class="secondary" :disabled="locked || !native || settingsDirty" @click="checkTools">{{ t('settings.check') }}</button></div><div v-for="tool in diagnostics" :key="tool.name" class="diagnostic"><span class="tool-status" :class="{ available: tool.available }"><AppIcon v-if="tool.available" name="check" :size="13" /><span v-else>!</span></span><strong>{{ tool.name }}</strong><pre>{{ formatAppMessage(tool.detail) }}</pre></div><p v-if="!diagnostics.length" class="empty-note">{{ t('settings.diagnosticsEmpty') }}</p></section>
        </template>
        <template v-else>
          <div class="page-heading"><div><span class="eyebrow accent">{{ t('history.eyebrow') }}</span><h1>{{ t('nav.history') }}</h1><p>{{ t('history.description') }}</p></div><span class="badge">{{ t('history.count', { count: data.history.length }) }}</span></div>
          <div v-if="!data.history.length" class="history-empty"><AppIcon name="clock" :size="38" /><h2>{{ t('history.empty') }}</h2><p>{{ t('history.emptyDescription') }}</p></div>
          <div v-else class="history-table-wrap"><table><thead><tr><th>{{ t('history.site') }}</th><th>{{ t('history.operation') }}</th><th>{{ t('history.targets') }}</th><th>{{ t('history.result') }}</th></tr></thead><tbody><tr v-for="entry in data.history" :key="entry.id"><td><strong>{{ entry.siteName }}</strong><small>{{ date(entry.startedAt) }}</small></td><td><span class="history-direction">{{ entry.request.direction.toUpperCase() }}</span><small>{{ entry.request.simulate ? t('common.simulation') : t('common.sync') }}</small></td><td>{{ entry.request.environment }}<small>{{ entry.request.targets.map(id => t(`target.${id}`)).join(', ') }}</small></td><td><span class="result-badge" :class="entry.status">{{ statusText(entry.status) }}</span><small>{{ t('common.exit', { code: entry.exitCode ?? '—' }) }}</small></td></tr></tbody></table></div>
          <p class="operation-note">{{ t('history.privacy') }}</p>
        </template>
      </div>
      <LogPanel v-if="view === 'sync' && site || running" :logs="logs" :result="lastResult" :run-error="runError" :running="running" :stopping="stopping" @stop="stop" />
      <footer class="statusbar"><span role="status"><span class="status-dot" :class="{ working: locked }" />{{ running ? (stopping ? t('footer.stopping') : t('footer.running')) : (busy ? t(busy) : t('common.ready')) }}</span><span>MoveDock <span class="version">0.2.1</span></span></footer>
    </main>
  </div>
</template>
