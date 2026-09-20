<template>
  <div class="interface-editor">
    <div class="interface-create__section">
      <div class="interface-url-row">
        <div class="interface-url-bar" :class="{ 'interface-url-bar--grpc': isGrpc }">
          <div
            class="interface-url-bar__selects"
            :class="{ 'interface-url-bar__selects--single': isProtocolOnly }"
          >
            <div class="interface-url-bar__protocol-wrap">
              <MenuSelect
                :model-value="protocol"
                class="interface-url-bar__protocol"
                :options="protocolOptions"
                fit-menu
                :menu-width="isProtocolOnly ? 180 : 108"
                @update:model-value="onProtocolChange"
              />
            </div>
            <div v-if="!isProtocolOnly" class="interface-url-bar__method-wrap">
              <MethodPicker
                v-model="method"
                class="interface-url-bar__method"
                :options="methodOptions"
              />
            </div>
          </div>
          <div class="interface-url-bar__url-wrap">
            <div
              ref="urlHighlightRef"
              class="interface-url-bar__url-highlight"
              aria-hidden="true"
              v-html="highlightedUrlHtml"
            />
            <input
              ref="urlInputRef"
              v-model="displayUrl"
              class="interface-url-bar__url native-input"
              placeholder=""
              spellcheck="false"
              @scroll="syncUrlHighlightScroll"
              @paste="onUrlPaste"
              @input="onUrlInput"
              @keydown="onUrlKeydown"
              @mousemove="onUrlInputMouseMove"
              @mouseleave="onUrlInputMouseLeave"
            />
            <Teleport to="body">
              <div
                v-if="urlVarTip"
                class="url-var-tip"
                :class="{ 'url-var-tip--missing': urlVarTip.missing }"
                :style="urlVarTipStyle"
                @mouseenter="onUrlVarTipEnter"
                @mouseleave="onUrlVarTipLeave"
              >
                <div class="url-var-tip__key">{{ urlVarTip.key }}</div>
                <div class="url-var-tip__value">{{ urlVarTip.value }}</div>
              </div>
            </Teleport>
          </div>
          <div v-if="isGrpc" class="interface-url-bar__rpc-wrap">
            <MenuSelect
              :model-value="grpcSelectedRpc || null"
              class="interface-url-bar__rpc"
              :options="grpcRpcOptions"
              :placeholder="t('grpc.selectMethod')"
              fit-menu
              :menu-width="280"
              @update:model-value="onGrpcRpcChange"
            />
          </div>
        </div>
        <div class="interface-url-actions">
          <template v-if="isWs">
            <button
              class="btn interface-url-actions__send"
              type="button"
              :class="wsConnected ? 'btn-danger' : 'btn-primary'"
              :disabled="wsConnecting || wsDisconnecting"
              @click="onWsConnectToggle"
            >
              {{
                wsConnecting
                  ? t('request.connecting')
                  : wsDisconnecting
                    ? t('request.disconnecting')
                    : wsConnected
                      ? t('request.disconnect')
                      : t('request.connect')
              }}
            </button>
            <button
              class="btn btn-primary interface-url-actions__send"
              type="button"
              :disabled="!wsConnected || wsSending || !wsCanSend"
              @click="onWsSend"
            >
              {{ wsSending ? t('request.sending') : t('request.send') }}
            </button>
          </template>
          <button
            v-else
            class="btn btn-primary interface-url-actions__send"
            type="button"
            :class="{ 'is-busy': sending }"
            :disabled="sending"
            @click="onSend"
          >
            {{ sending ? t('request.sending') : isGrpc ? t('request.invoke') : t('request.send') }}
          </button>
          <button
            class="btn interface-url-actions__save"
            type="button"
            :class="{
              'is-busy': saveUi === 'saving',
              'is-saved': saveUi === 'saved',
            }"
            :disabled="saveUi === 'saving'"
            @click="onSave"
          >
            <span class="interface-url-actions__save-label">
              <template v-if="saveUi === 'saved'">
                <span class="interface-url-actions__save-check" aria-hidden="true">✓</span>
                {{ t('common.saved') }}
              </template>
              <template v-else>
                {{ t('common.save') }}
              </template>
            </span>
          </button>
          <button v-if="mode === 'create'" class="btn" type="button" @click="$emit('cancel')">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>

      <div class="interface-meta-row">
        <MenuSelect
          v-model="folderId"
          class="interface-meta-row__folder"
          :options="folderOptions"
          :placeholder="t('create.selectFolder')"
        />
        <input
          v-model="name"
          class="interface-meta-row__name native-input"
          :placeholder="t('create.apiNamePlaceholder')"
        />
      </div>
    </div>

    <div
      ref="splitContainerRef"
      class="interface-work-split"
      :class="{ 'interface-work-split--grpc': isGrpc }"
    >
      <template v-if="isGrpc">
        <GrpcMethodList
          class="grpc-method-host"
          :style="{ width: `${grpcListWidth}px` }"
          :proto-name="grpcProtoName"
          :services="grpcServices"
          :selected-rpc="grpcSelectedRpc"
          @select="selectGrpcRpc"
          @import="pickProtoFile"
        />
        <div
          class="grpc-list-handle"
          :class="{ 'grpc-list-handle--active': grpcListDragging }"
          @mousedown="startGrpcListDrag"
        />
      </template>
      <div class="interface-work-main">
      <section class="interface-request-pane">
        <div class="interface-request-tabs">
          <button
            v-for="tab in requestTabs"
            :key="tab.id"
            type="button"
            class="interface-request-tabs__tab"
            :class="{ 'interface-request-tabs__tab--active': requestTab === tab.id }"
            @click="requestTab = tab.id"
          >
            {{ t(tab.labelKey) }}
            <span
              v-if="tab.id === 'headers' && headerTabCount"
              class="interface-request-tabs__badge"
            >
              ({{ headerTabCount }})
            </span>
          </button>
        </div>

        <div
          class="interface-request-content"
          :class="{ 'interface-request-content--fill': isGrpc && requestTab === 'message' }"
        >
          <div
            v-if="requestTab === 'params'"
            ref="paramTableEl"
            class="param-table param-table--resizable-wrap"
          >
            <ParamTableColumnResizer
              :columns="paramGridColumns"
              :handles="[false, 'key', 'value', false, false]"
              @resize="startTableColResize"
            />
            <div class="param-table__row param-table__row--head" :style="{ gridTemplateColumns: paramGridColumns }">
              <label class="param-table__check">
                <input
                  type="checkbox"
                  :checked="queryParamsSelectAll.checked"
                  :ref="(el) => setCheckboxIndeterminate(el, queryParamsSelectAll.indeterminate)"
                  :title="t('common.selectAll')"
                  :aria-label="t('common.selectAll')"
                  @change="setAllRowsEnabled(queryParams, ($event.target as HTMLInputElement).checked)"
                />
              </label>
              <span class="param-table__head-cell">{{ t('create.param.name') }}</span>
              <span class="param-table__head-cell">{{ t('common.value') }}</span>
              <span class="param-table__head-cell">{{ t('create.param.desc') }}</span>
              <span class="param-table__head-action" />
            </div>
            <div
              v-for="(row, idx) in queryParams"
              :key="idx"
              class="param-table__row"
              :class="{ 'param-table__row--off': row.enabled === false }"
              :style="{ gridTemplateColumns: paramGridColumns }"
            >
              <label class="param-table__check">
                <input v-model="row.enabled" type="checkbox" />
              </label>
              <input
                v-model="row.name"
                class="param-table__cell"
                :placeholder="t('common.key')"
                @input="onQueryParamChange(idx)"
              />
              <input
                v-model="row.example"
                class="param-table__cell"
                :placeholder="t('common.value')"
                @input="onQueryParamChange(idx)"
              />
              <input
                v-model="row.description"
                class="param-table__cell"
                :placeholder="t('create.param.desc')"
                @input="onQueryParamChange(idx)"
              />
              <button
                type="button"
                class="param-table__del"
                :disabled="queryParams.length <= 1"
                @click="removeQueryRow(idx)"
              >
                ×
              </button>
            </div>
          </div>

          <div v-else-if="requestTab === 'authorization'" class="auth-pane">
            <div class="auth-pane__layout">
              <div class="auth-pane__left">
                <span class="auth-pane__label">{{ t('request.authType') }}</span>
                <MenuSelect
                  v-model="authType"
                  compact
                  class="auth-pane__type"
                  :menu-width="168"
                  :options="authTypeOptions"
                />
                <p v-if="authType !== 'none'" class="auth-pane__hint muted">{{ authTypeHint }}</p>
              </div>
              <div class="auth-pane__divider" />
              <div class="auth-pane__right">
                <template v-if="authType === 'none'">
                  <span class="auth-pane__label auth-pane__label--spacer" aria-hidden="true">&nbsp;</span>
                  <p class="auth-pane__hint auth-pane__hint--none muted">{{ t('request.authNoneHint') }}</p>
                </template>
                <template v-else-if="authType === 'bearer'">
                  <span class="auth-pane__label">{{ t('request.authToken') }}</span>
                  <input
                    v-model="authBearerToken"
                    class="native-input auth-pane__input"
                    :placeholder="t('request.authToken')"
                  />
                </template>
                <template v-else-if="authType === 'basic'">
                  <span class="auth-pane__label">{{ t('request.authUsername') }}</span>
                  <input
                    v-model="authBasicUser"
                    class="native-input auth-pane__input"
                    :placeholder="t('request.authUsername')"
                  />
                  <span class="auth-pane__label">{{ t('request.authPassword') }}</span>
                  <input
                    v-model="authBasicPassword"
                    class="native-input auth-pane__input"
                    type="password"
                    :placeholder="t('request.authPassword')"
                  />
                </template>
              </div>
            </div>
          </div>

          <div
            v-else-if="requestTab === 'headers' || requestTab === 'metadata'"
            class="headers-pane"
          >
            <button
              v-if="requestTab === 'headers' && hiddenHeaders.length"
              type="button"
              class="hidden-headers-toggle"
              :class="{ 'hidden-headers-toggle--open': showHiddenHeaders }"
              :aria-expanded="showHiddenHeaders"
              @click="showHiddenHeaders = !showHiddenHeaders"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                <path
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M1.5 12s3.8-7.5 10.5-7.5S22.5 12 22.5 12 18.7 19.5 12 19.5 1.5 12 1.5 12z"
                />
                <circle
                  cx="12"
                  cy="12"
                  r="3.2"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                />
              </svg>
              {{ t('request.hiddenHeaders', { n: hiddenHeaders.length }) }}
            </button>
            <div ref="paramTableEl" class="param-table param-table--resizable-wrap">
            <ParamTableColumnResizer
              :columns="paramGridColumns"
              :handles="[false, 'key', 'value', false, false]"
              @resize="startTableColResize"
            />
            <div class="param-table__row param-table__row--head" :style="{ gridTemplateColumns: paramGridColumns }">
              <label class="param-table__check">
                <input
                  type="checkbox"
                  :checked="requestHeadersSelectAll.checked"
                  :ref="(el) => setCheckboxIndeterminate(el, requestHeadersSelectAll.indeterminate)"
                  :title="t('common.selectAll')"
                  :aria-label="t('common.selectAll')"
                  @change="setAllRowsEnabled(requestHeaders, ($event.target as HTMLInputElement).checked)"
                />
              </label>
              <span class="param-table__head-cell">{{ t('create.param.name') }}</span>
              <span class="param-table__head-cell">{{ t('common.value') }}</span>
              <span class="param-table__head-cell">{{ t('create.param.desc') }}</span>
              <span class="param-table__head-action" />
            </div>
            <div
              v-for="row in showHiddenHeaders ? hiddenHeaders : []"
              :key="`auto-${row.key}`"
              class="param-table__row param-table__row--auto"
              :class="{ 'param-table__row--off': !autoHeaderEnabled(row.key) }"
              :style="{ gridTemplateColumns: paramGridColumns }"
            >
              <label class="param-table__check">
                <input
                  type="checkbox"
                  :checked="autoHeaderEnabled(row.key)"
                  @change="setAutoHeaderEnabled(row.key, ($event.target as HTMLInputElement).checked)"
                />
              </label>
              <div class="param-table__auto-key">
                <span>{{ row.key }}</span>
                <span
                  class="param-table__auto-info"
                  :title="t('request.hiddenHeaderHint')"
                >i</span>
              </div>
              <div class="param-table__auto-value" :title="row.value">{{ row.value }}</div>
              <div class="param-table__auto-desc" />
              <div class="param-table__auto-spacer" />
            </div>
            <div
              v-for="(row, idx) in requestHeaders"
              :key="idx"
              class="param-table__row"
              :class="{ 'param-table__row--off': row.enabled === false }"
              :style="{ gridTemplateColumns: paramGridColumns }"
            >
              <label class="param-table__check">
                <input v-model="row.enabled" type="checkbox" />
              </label>
              <ComboInput
                v-model="row.name"
                class="param-table__combo"
                :options="headerKeySuggestions"
                :placeholder="t('common.key')"
                @input="onHeaderChange(idx)"
              />
              <input
                v-model="row.example"
                class="param-table__cell"
                :placeholder="t('common.value')"
                @input="onHeaderChange(idx)"
              />
              <input
                v-model="row.description"
                class="param-table__cell"
                :placeholder="t('create.param.desc')"
                @input="onHeaderChange(idx)"
              />
              <button
                type="button"
                class="param-table__del"
                :disabled="requestHeaders.length <= 1"
                @click="removeHeaderRow(idx)"
              >
                ×
              </button>
            </div>
            </div>
          </div>

          <div v-else-if="requestTab === 'body'" class="body-pane">
            <div class="body-format-row">
              <div class="body-format-switch">
                <button
                  v-for="fmt in bodyFormats"
                  :key="fmt.value"
                  type="button"
                  class="body-format-switch__btn"
                  :class="{ 'body-format-switch__btn--active': bodyFormat === fmt.value }"
                  @click="bodyFormat = fmt.value"
                >
                  {{ fmt.label }}
                </button>
              </div>
              <button
                v-if="bodyFormat === 'raw'"
                type="button"
                class="body-format-action"
                @click="formatBodyRaw"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                  <path
                    fill="currentColor"
                    d="M7 5h10v2H7V5zm0 4h10v2H7V9zm0 4h7v2H7v-2zm-1 6l-4-4h3v-4h2v4h3l-4 4z"
                  />
                </svg>
                {{ t('request.rawFormat') }}
              </button>
              <button
                v-else-if="bodyFormat === 'form-data'"
                type="button"
                class="body-format-action"
                @click="importJsonDialogOpen = true"
              >
                <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                  <path
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M12 3v10m0 0-4-4m4 4 4-4M5 15v3a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-3"
                  />
                </svg>
                {{ t('request.importJson') }}
              </button>
            </div>

            <p v-if="bodyFormat === 'none'" class="body-none-hint muted">
              {{ t('request.bodyNoneHint') }}
            </p>

            <div
              v-else-if="bodyFormat === 'form-data'"
              ref="paramTableEl"
              class="param-table param-table--resizable-wrap"
            >
              <ParamTableColumnResizer
                :columns="formDataGridColumns"
                :handles="[false, 'key', 'type', 'value', false, false]"
                @resize="startTableColResize"
              />
              <div
                class="param-table__row param-table__row--head"
                :style="{ gridTemplateColumns: formDataGridColumns }"
              >
                <label class="param-table__check">
                  <input
                    type="checkbox"
                    :checked="formDataSelectAll.checked"
                    :ref="(el) => setCheckboxIndeterminate(el, formDataSelectAll.indeterminate)"
                    :title="t('common.selectAll')"
                    :aria-label="t('common.selectAll')"
                    @change="setAllRowsEnabled(formDataFields, ($event.target as HTMLInputElement).checked)"
                  />
                </label>
                <span class="param-table__head-cell">{{ t('create.param.name') }}</span>
                <span class="param-table__head-cell">{{ t('create.param.type') }}</span>
                <span class="param-table__head-cell">{{ t('common.value') }}</span>
                <span class="param-table__head-cell">{{ t('create.param.desc') }}</span>
                <span class="param-table__head-action" />
              </div>
              <div
                v-for="(row, idx) in formDataFields"
                :key="idx"
                class="param-table__row"
                :class="{ 'param-table__row--off': row.enabled === false }"
                :style="{ gridTemplateColumns: formDataGridColumns }"
              >
                <label class="param-table__check">
                  <input v-model="row.enabled" type="checkbox" />
                </label>
                <input
                  v-model="row.name"
                  class="param-table__cell"
                  :placeholder="t('common.key')"
                  @input="onFormDataChange(idx)"
                />
                <div class="param-table__type">
                  <div class="param-table__type-select">
                    <MenuSelect
                      v-model="row.type"
                      compact
                      plain
                      fit-menu
                      dense
                      menu-align="left"
                      :menu-min-width="76"
                      :options="formDataTypeOptions"
                      @update:model-value="onFormDataTypeChange(idx)"
                    />
                  </div>
                </div>
                <div class="param-table__value">
                  <div v-if="row.type !== 'file'" class="param-table__value-shell">
                    <textarea
                      v-model="row.example"
                      class="param-table__cell param-table__cell--inner param-table__value-input"
                      rows="1"
                      :placeholder="t('common.value')"
                      @focus="onFormDataValueFocus"
                      @blur="onFormDataValueBlur"
                      @input="onFormDataValueInput(idx, $event)"
                    />
                  </div>
                  <label v-else class="param-table__file">
                    <input
                      type="file"
                      class="param-table__file-input"
                      @change="onFormDataFileChange(idx, $event)"
                    />
                    <span class="param-table__file-name" :class="{ filled: !!row.example }">
                      {{ row.example || t('request.formDataSelectFile') }}
                    </span>
                  </label>
                </div>
                <input
                  v-model="row.description"
                  class="param-table__cell"
                  :placeholder="t('create.param.desc')"
                  @input="onFormDataChange(idx)"
                />
                <div class="param-table__actions">
                  <FormDataDictPicker
                    :param-name="row.name"
                    :entries="workspaceDictionaries"
                    :matched="formDataRowDictMatched(row.name)"
                    @before-open="() => refreshWorkspaceDictionaries(true)"
                    @pick="(entry) => applyDictionaryToFormDataRow(idx, entry)"
                    @apply-all="applyDictionaryDescriptionsToAllFormData"
                  />
                  <button
                    type="button"
                    class="param-table__del"
                    :disabled="formDataFields.length <= 1"
                    @click="removeFormDataRow(idx)"
                  >
                    ×
                  </button>
                </div>
              </div>
            </div>

            <RequestBodyRawEditor
              v-else-if="bodyFormat === 'raw'"
              ref="rawBodyEditorRef"
              v-model="bodyRaw"
              :placeholder="t('create.bodyPlaceholder')"
            />
          </div>

          <div v-else-if="requestTab === 'message'" class="body-pane">
            <div class="body-format-row body-format-row--grpc">
              <div class="body-format-switch">
                <button type="button" class="body-format-switch__btn" @click="autoGenerateGrpcMessage">
                  {{ t('grpc.autoGenerate') }}
                </button>
              </div>
              <button type="button" class="body-format-action" @click="formatBodyRaw">
                <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
                  <path
                    fill="currentColor"
                    d="M7 5h10v2H7V5zm0 4h10v2H7V9zm0 4h7v2H7v-2zm-1 6l-4-4h3v-4h2v4h3l-4 4z"
                  />
                </svg>
                {{ t('request.rawFormat') }}
              </button>
            </div>
            <RequestBodyRawEditor
              ref="rawBodyEditorRef"
              v-model="bodyRaw"
              :placeholder="t('grpc.messagePlaceholder')"
            />
          </div>
        </div>
      </section>

      <div
        class="interface-split-handle"
        :class="{ 'interface-split-handle--active': responseDragging }"
        @mousedown="startResponseDrag"
      />

      <section class="interface-response-pane" :style="{ height: `${responseHeight}px` }">
        <div class="interface-response-bar">
          <div class="interface-response-tabs">
            <button
              type="button"
              class="interface-response-tabs__tab"
              :class="{ 'interface-response-tabs__tab--active': responseTab === 'body' }"
              @click="responseTab = 'body'"
            >
              {{ isWs ? t('response.messages') : isGrpc ? t('request.response') : t('response.body') }}
              <span v-if="isWs && wsMessages.length" class="interface-response-tabs__badge">
                {{ wsMessages.length }}
              </span>
            </button>
            <button
              type="button"
              class="interface-response-tabs__tab"
              :class="{ 'interface-response-tabs__tab--active': responseTab === 'headers' }"
              @click="responseTab = 'headers'"
            >
              {{ isGrpc ? t('request.metadata') : t('response.headers') }}
              <span
                v-if="(isWs ? wsResponseHeaders : response?.headers)?.length"
                class="interface-response-tabs__badge"
              >
                {{ (isWs ? wsResponseHeaders : response?.headers)?.length }}
              </span>
            </button>
          </div>
          <div v-if="isWs" class="interface-response-bar__right">
            <div class="interface-response-meta">
              <span
                class="interface-response-meta__status"
                :class="wsConnected ? 'ok' : wsConnectError ? 'err' : ''"
              >
                {{
                  wsConnected
                    ? t('request.wsConnected')
                    : wsConnectError
                      ? t('request.wsError')
                      : t('request.wsIdle')
                }}
              </span>
              <template v-if="wsStatusText">
                <span class="interface-response-meta__sep" aria-hidden="true" />
                <span class="interface-response-meta__item">{{ wsStatusText }}</span>
              </template>
              <button
                v-if="wsMessages.length"
                type="button"
                class="btn interface-response-clear"
                @click="clearWsMessages"
              >
                {{ t('response.clearMessages') }}
              </button>
            </div>
          </div>
          <div v-else-if="response && !sending" class="interface-response-bar__right">
            <div
              v-if="responseTab === 'body' && !isSendError"
              class="body-format-switch interface-response-type-switch"
            >
              <button
                v-for="fmt in responseBodyTypes"
                :key="fmt.value"
                type="button"
                class="body-format-switch__btn"
                :class="{ 'body-format-switch__btn--active': responseBodyType === fmt.value }"
                @click="responseBodyType = fmt.value"
              >
                {{ fmt.label }}
              </button>
            </div>
            <div class="interface-response-meta">
              <span class="interface-response-meta__status" :class="responseStatusClass">
                {{ response.status ? `${response.status} ${response.status_text}` : response.status_text }}
              </span>
              <span class="interface-response-meta__sep" aria-hidden="true" />
              <span class="interface-response-meta__item">{{ response.duration_ms }} ms</span>
              <span class="interface-response-meta__sep" aria-hidden="true" />
              <span class="interface-response-meta__item">{{ responseBodySize }}</span>
            </div>
          </div>
        </div>

        <div class="interface-response-content">
          <template v-if="isWs">
            <div v-if="responseTab === 'body'" class="ws-message-list" ref="wsMessageListRef">
              <p v-if="!wsMessages.length" class="interface-response-empty muted">
                {{ t('response.wsEmpty') }}
              </p>
              <div
                v-for="msg in wsMessages"
                :key="msg.id"
                class="ws-message"
                :class="`ws-message--${msg.direction}`"
              >
                <div class="ws-message__meta">
                  <span class="ws-message__dir">{{ wsDirectionLabel(msg.direction) }}</span>
                  <span class="ws-message__type">{{ msg.messageType }}</span>
                  <span class="ws-message__time">{{ formatWsTime(msg.at) }}</span>
                </div>
                <pre class="ws-message__body">{{ msg.data }}</pre>
              </div>
            </div>
            <div v-else class="interface-response-headers-table">
              <div class="interface-response-headers-table__head">
                <span>{{ t('common.key') }}</span>
                <span>{{ t('common.value') }}</span>
              </div>
              <div
                v-for="(h, i) in wsResponseHeaders"
                :key="`${h.key}-${i}`"
                class="interface-response-headers-table__row"
              >
                <span class="interface-response-headers-table__key">{{ h.key }}</span>
                <span class="interface-response-headers-table__value">{{ h.value }}</span>
              </div>
              <p v-if="!wsResponseHeaders.length" class="interface-response-empty muted">
                {{ t('response.empty') }}
              </p>
            </div>
          </template>
          <div v-else-if="sending" class="interface-response-loading" role="status" aria-live="polite">
            <div class="interface-response-loading__spinner" aria-hidden="true" />
            <p class="interface-response-loading__text">{{ t('request.sending') }}</p>
          </div>
          <ResponseErrorState v-else-if="isSendError" :message="response?.body ?? ''" />
          <template v-else-if="response">
            <RequestBodyRawEditor
              v-if="responseTab === 'body'"
              ref="responseBodyEditorRef"
              :model-value="responseDisplayBody"
              :content-type="responseBodyType"
              readonly
              class="interface-response-raw"
              @scroll="onResponseBodyScroll"
            />
            <div v-else class="interface-response-headers-table">
              <div class="interface-response-headers-table__head">
                <span>{{ t('common.key') }}</span>
                <span>{{ t('common.value') }}</span>
              </div>
              <div
                v-for="(h, i) in response.headers"
                :key="`${h.key}-${i}`"
                class="interface-response-headers-table__row"
              >
                <span class="interface-response-headers-table__key">{{ h.key }}</span>
                <span class="interface-response-headers-table__value">{{ h.value }}</span>
              </div>
            </div>
          </template>
          <p v-else class="interface-response-empty muted">{{ t('response.empty') }}</p>
        </div>
      </section>
      </div>
    </div>
    <input
      ref="protoFileRef"
      type="file"
      accept=".proto,text/plain"
      class="grpc-file-input"
      @change="onProtoFileChange"
    />
    <ConfirmDialog
      v-model:open="validationDialogOpen"
      :title="validationDialogTitle"
      :message="validationDialogMessage"
      variant="primary"
      :show-cancel="false"
      @confirm="validationDialogOpen = false"
    />
    <ImportJsonDialog v-model:open="importJsonDialogOpen" @apply="onImportFormDataJson" />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from '../i18n'
import { useContainerBoundedResize } from '../composables/useDragResize'
import { useColumnResize } from '../composables/useColumnResize'
import type { ParamRow } from './ParamTable.vue'
import MenuSelect from './MenuSelect.vue'
import MethodPicker from './MethodPicker.vue'
import ComboInput from './ComboInput.vue'
import ParamTableColumnResizer from './ParamTableColumnResizer.vue'
import RequestBodyRawEditor from './RequestBodyRawEditor.vue'
import ResponseErrorState from './ResponseErrorState.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import ImportJsonDialog from './ImportJsonDialog.vue'
import FormDataDictPicker from './FormDataDictPicker.vue'
import GrpcMethodList from './GrpcMethodList.vue'
import { formatJsonLenient, stripJsonComments } from '../utils/raw-json-utils'
import { applyFormDataJsonImport, type FormDataImportMode } from '../utils/form-data-json-import'
import {
  applyDictDescriptionsToFormData,
  paramHasDictMatch,
} from '../utils/dictionary-match'
import { looksLikeCurl, parseCurl, type ParsedCurl } from '../utils/parse-curl'
import { buildAutoHeaders, isAutoHeaderEnabled, sendableAutoHeaders } from '../utils/auto-headers'
import {
  compactGrpcMetadata,
  fillGrpcMessageSamples,
  findGrpcMethod,
  firstRpcPath,
  normalizeGrpcMetadata,
  parseGrpcCatalog,
  type GrpcCatalog,
  type GrpcService,
} from '../utils/grpc'
import {
  isKnownProtocol,
  isProtocolOnlySelect,
  methodsForProtocol,
  normalizeProtocol,
  protocolFamily,
  protocolLabel,
  switchableProtocols,
  type Protocol,
} from '../utils/http-method'
import { useToast } from '../composables/useToast'
import { useWorkspaceContext } from '../composables/useWorkspaceContext'
import { createTextUndo, matchUndoRedoKey } from '../composables/useTextUndo'
import type { DictionaryEntryRow, FormDataSendField, HeaderPair, SendResponse } from '../types'

export type FolderOption = { id: number; name: string; parent_id: number; label: string }

type QueryParamRow = ParamRow & { enabled: boolean }

type WsMessageItem = {
  id: string
  direction: 'in' | 'out' | 'system'
  kind: string
  messageType: string
  data: string
  at: number
}

type WsSessionEventPayload = {
  session_id: string
  kind: 'connected' | 'sent' | 'message' | 'error' | 'closed'
  direction: string
  data: string
  message_type: string
  at_ms: number
}

type WsConnectResult = {
  session_id: string
  status: number
  status_text: string
  headers: HeaderPair[]
  url: string
}
type HeaderRow = ParamRow & { enabled: boolean }
type BodyFormat = 'none' | 'form-data' | 'raw'
type FormDataRow = ParamRow & { enabled: boolean; fileData?: string; fileName?: string }
type RequestTabId = 'params' | 'authorization' | 'headers' | 'body' | 'message' | 'metadata'

const props = defineProps<{
  mode: 'create' | 'edit'
  workspaceId: number
  projectId: number
  interfaceId?: number | null
  initial?: {
    name?: string
    method?: string
    url?: string
    folder_id?: number
    status?: number
    detail?: Record<string, unknown> | null
    lastResponse?: SendResponse | null
    requestPaneTab?: RequestTabId
    responseScrollTop?: number
  } | null
}>()

const emit = defineEmits<{
  saved: [id: number]
  cancel: []
  'state-change': []
  'editor-ready': []
}>()

const { t } = useI18n()
const { showToast } = useToast()
const { environmentId, envVarsRevision } = useWorkspaceContext()
const folderOptions = computed(() => [
  { value: 0, label: t('create.noFolder') },
  ...folders.value.map((f) => ({ value: f.id, label: f.label })),
])
const envVarKeys = ref<Set<string>>(new Set())
const envVarMap = ref<Record<string, string>>({})
const urlHighlightRef = ref<HTMLElement | null>(null)
const urlInputRef = ref<HTMLInputElement | null>(null)
const urlUndo = createTextUndo()
let applyingUrlUndo = false

type UrlVarTip = {
  key: string
  value: string
  missing: boolean
  left: number
  top: number
}
const urlVarTip = ref<UrlVarTip | null>(null)
const urlVarTipStyle = computed(() => {
  const tip = urlVarTip.value
  if (!tip) return {}
  return {
    left: `${tip.left}px`,
    top: `${tip.top}px`,
  }
})
let urlVarTipHideTimer: ReturnType<typeof setTimeout> | null = null
let urlVarTipShowTimer: ReturnType<typeof setTimeout> | null = null
const URL_VAR_TIP_SHOW_DELAY_MS = 420
const URL_VAR_TIP_HIDE_DELAY_MS = 200

const ENV_VAR_RE = /\{\{\s*([^{}]+?)\s*\}\}/g

type UrlVarRange = { start: number; end: number; key: string; ok: boolean }

type UrlVarHitBox = UrlVarRange & { left: number; right: number; bottom: number }

let urlInputMeasureCtx: CanvasRenderingContext2D | null = null
let urlInputMeasureFont = ''
let urlVarHitBoxes: UrlVarHitBox[] = []
let urlVarHitBoxesFor = ''
let urlVarHoverRaf = 0
let urlVarHoverKey: string | null = null
let urlVarTipPinned = false

function findUrlVarRanges(text: string): UrlVarRange[] {
  const ranges: UrlVarRange[] = []
  ENV_VAR_RE.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = ENV_VAR_RE.exec(text))) {
    const key = match[1].trim()
    ranges.push({
      start: match.index,
      end: match.index + match[0].length,
      key,
      ok: envVarKeys.value.has(key),
    })
  }
  return ranges
}

function urlInputTextCtx(input: HTMLInputElement): CanvasRenderingContext2D {
  if (!urlInputMeasureCtx) {
    const canvas = document.createElement('canvas')
    urlInputMeasureCtx = canvas.getContext('2d')!
  }
  const style = getComputedStyle(input)
  const font = `${style.fontStyle} ${style.fontVariant} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`
  if (font !== urlInputMeasureFont) {
    urlInputMeasureCtx.font = font
    urlInputMeasureFont = font
  }
  return urlInputMeasureCtx
}

function urlInputTextWidth(input: HTMLInputElement, text: string): number {
  return urlInputTextCtx(input).measureText(text).width
}

function rebuildUrlVarHitBoxes() {
  const input = urlInputRef.value
  const text = displayUrl.value
  urlVarHitBoxesFor = text
  if (!input || !text) {
    urlVarHitBoxes = []
    return
  }
  const rect = input.getBoundingClientRect()
  const padL = parseFloat(getComputedStyle(input).paddingLeft) || 0
  const bottom = rect.bottom + 6
  const scrollLeft = input.scrollLeft
  urlVarHitBoxes = findUrlVarRanges(text).map((r) => {
    const left =
      rect.left + padL + urlInputTextWidth(input, text.slice(0, r.start)) - scrollLeft
    const right =
      rect.left + padL + urlInputTextWidth(input, text.slice(0, r.end)) - scrollLeft
    return { ...r, left, right, bottom }
  })
}

function clearUrlVarTipHideTimer() {
  if (urlVarTipHideTimer == null) return
  clearTimeout(urlVarTipHideTimer)
  urlVarTipHideTimer = null
}

function clearUrlVarTipShowTimer() {
  if (urlVarTipShowTimer == null) return
  clearTimeout(urlVarTipShowTimer)
  urlVarTipShowTimer = null
}

function hideUrlVarTip() {
  clearUrlVarTipHideTimer()
  clearUrlVarTipShowTimer()
  if (urlVarHoverRaf) {
    cancelAnimationFrame(urlVarHoverRaf)
    urlVarHoverRaf = 0
  }
  urlVarTipPinned = false
  urlVarTip.value = null
  urlVarHoverKey = null
}

function scheduleHideUrlVarTip() {
  if (urlVarTipPinned) return
  clearUrlVarTipShowTimer()
  if (!urlVarTip.value) return
  clearUrlVarTipHideTimer()
  urlVarTipHideTimer = setTimeout(() => {
    urlVarTipHideTimer = null
    if (urlVarTipPinned) return
    urlVarTip.value = null
    urlVarHoverKey = null
  }, URL_VAR_TIP_HIDE_DELAY_MS)
}

function openUrlVarTip(hit: UrlVarHitBox) {
  clearUrlVarTipHideTimer()
  const rawValue = envVarMap.value[hit.key]
  let value: string
  if (!hit.ok) value = t('request.envVarUndefined')
  else if (rawValue == null || rawValue === '') value = t('request.envVarEmpty')
  else value = rawValue
  urlVarTip.value = {
    key: hit.key,
    value,
    missing: !hit.ok,
    left: (hit.left + hit.right) / 2,
    top: hit.bottom,
  }
}

function scheduleShowUrlVarTip(hit: UrlVarHitBox) {
  clearUrlVarTipHideTimer()
  // Already showing this key — keep it.
  if (urlVarTip.value?.key === hit.key) return
  clearUrlVarTipShowTimer()
  urlVarTipShowTimer = setTimeout(() => {
    urlVarTipShowTimer = null
    openUrlVarTip(hit)
  }, URL_VAR_TIP_SHOW_DELAY_MS)
}

function onUrlVarTipEnter() {
  urlVarTipPinned = true
  clearUrlVarTipHideTimer()
  clearUrlVarTipShowTimer()
}

function onUrlVarTipLeave() {
  urlVarTipPinned = false
  scheduleHideUrlVarTip()
}

function onUrlInputMouseMove(event: MouseEvent) {
  // Dragging to select text — skip heavy work (was freezing the UI).
  if (event.buttons !== 0) {
    if (urlVarHoverKey !== null || urlVarTip.value || urlVarTipShowTimer != null) {
      const input = urlInputRef.value
      if (input) input.style.cursor = ''
      hideUrlVarTip()
    }
    return
  }

  const clientX = event.clientX
  if (urlVarHoverRaf) cancelAnimationFrame(urlVarHoverRaf)
  urlVarHoverRaf = requestAnimationFrame(() => {
    urlVarHoverRaf = 0
    const input = urlInputRef.value
    if (!input) return

    const text = displayUrl.value
    if (urlVarHitBoxesFor !== text) rebuildUrlVarHitBoxes()

    const hit = urlVarHitBoxes.find((r) => clientX >= r.left && clientX < r.right)
    if (!hit) {
      if (urlVarHoverKey !== null || urlVarTipShowTimer != null) {
        urlVarHoverKey = null
        input.style.cursor = ''
        clearUrlVarTipShowTimer()
        scheduleHideUrlVarTip()
      }
      return
    }

    input.style.cursor = 'pointer'
    if (urlVarHoverKey === hit.key) {
      // Still over same var — cancel pending hide if any.
      clearUrlVarTipHideTimer()
      return
    }
    urlVarHoverKey = hit.key
    scheduleShowUrlVarTip(hit)
  })
}

function onUrlInputMouseLeave() {
  const input = urlInputRef.value
  if (input) input.style.cursor = ''
  urlVarHoverKey = null
  clearUrlVarTipShowTimer()
  // Delay hide so pointer can move onto the tip bubble.
  scheduleHideUrlVarTip()
}

function urlSnapshot() {
  const el = urlInputRef.value
  const text = displayUrl.value
  return {
    text,
    start: el?.selectionStart ?? text.length,
    end: el?.selectionEnd ?? text.length,
  }
}

function resetUrlUndo() {
  void nextTick(() => {
    urlUndo.reset(urlSnapshot())
  })
}

function onUrlInput() {
  if (applyingUrlUndo) return
  void nextTick(() => urlUndo.noteInput(urlSnapshot()))
}

function onUrlKeydown(event: KeyboardEvent) {
  const kind = matchUndoRedoKey(event)
  if (!kind) return
  event.preventDefault()
  const snap =
    kind === 'undo' ? urlUndo.undo(urlSnapshot()) : urlUndo.redo(urlSnapshot())
  if (!snap) return
  applyingUrlUndo = true
  parseUrlInput(snap.text)
  void nextTick(() => {
    const el = urlInputRef.value
    if (el) {
      const start = Math.max(0, Math.min(snap.start, el.value.length))
      const end = Math.max(0, Math.min(snap.end, el.value.length))
      try {
        el.setSelectionRange(start, end)
      } catch {
        /* ignore */
      }
    }
    applyingUrlUndo = false
  })
}

function escapeHtml(text: string) {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function applyEnvVars(input: string, vars: Record<string, string>) {
  if (!input || !Object.keys(vars).length) return input
  return input.replace(/\{\{\s*([^{}]+?)\s*\}\}/g, (match, key: string) => {
    const k = key.trim()
    return Object.prototype.hasOwnProperty.call(vars, k) ? vars[k] : match
  })
}

function syncUrlHighlightScroll(e: Event) {
  const el = e.target as HTMLInputElement
  if (urlHighlightRef.value) {
    urlHighlightRef.value.scrollLeft = el.scrollLeft
  }
  hideUrlVarTip()
  urlVarHoverKey = null
  urlVarHitBoxesFor = ''
}

async function refreshEnvVars() {
  if (!environmentId.value) {
    envVarKeys.value = new Set()
    envVarMap.value = {}
    return
  }
  try {
    const rows = await invoke<[string, string][]>('get_env_vars', {
      environmentId: environmentId.value,
    })
    const map: Record<string, string> = {}
    const keys = new Set<string>()
    for (const [k, v] of rows) {
      keys.add(k)
      map[k] = v
    }
    envVarKeys.value = keys
    envVarMap.value = map
  } catch {
    envVarKeys.value = new Set()
    envVarMap.value = {}
  }
}

watch([environmentId, envVarsRevision], () => {
  void refreshEnvVars()
})

function requestTabForProtocol(tab: RequestTabId | undefined, proto: Protocol): RequestTabId {
  const family = protocolFamily(proto)
  const current = tab ?? (family === 'grpc' ? 'message' : 'body')
  if (family === 'grpc') {
    if (current === 'headers' || current === 'metadata') return 'metadata'
    return 'message'
  }
  if (current === 'message') return 'body'
  if (current === 'metadata') return 'headers'
  if (family === 'ws' && current === 'authorization') return 'body'
  return current
}

const allRequestTabs: { id: RequestTabId; labelKey: string }[] = [
  { id: 'params', labelKey: 'request.params' },
  { id: 'authorization', labelKey: 'request.authorization' },
  { id: 'headers', labelKey: 'request.headers' },
  { id: 'body', labelKey: 'request.body' },
]

const grpcRequestTabs: { id: RequestTabId; labelKey: string }[] = [
  { id: 'message', labelKey: 'request.message' },
  { id: 'metadata', labelKey: 'request.metadata' },
]

const isGrpc = computed(() => protocolFamily(protocol.value) === 'grpc')
const isWs = computed(() => protocolFamily(protocol.value) === 'ws')

const requestTabs = computed(() => {
  if (isGrpc.value) return grpcRequestTabs
  if (isWs.value) {
    return allRequestTabs.filter((tab) => tab.id !== 'authorization')
  }
  return allRequestTabs
})

const headerKeySuggestions = [
  'Accept',
  'Accept-Language',
  'Authorization',
  'Cache-Control',
  'Connection',
  'Content-Type',
  'Cookie',
  'Host',
  'Origin',
  'Referer',
  'User-Agent',
  'X-Requested-With',
]

const authTypeOptions = computed(() => [
  { value: 'none', label: t('request.authNone') },
  { value: 'bearer', label: t('request.authBearer') },
  { value: 'basic', label: t('request.authBasic') },
])

const bodyFormats = computed(() => {
  if (isWs.value) {
    return [
      { value: 'none' as const, label: t('request.bodyNone') },
      { value: 'raw' as const, label: t('request.bodyRaw') },
    ]
  }
  return [
    { value: 'none' as const, label: t('request.bodyNone') },
    { value: 'form-data' as const, label: t('request.bodyFormData') },
    { value: 'raw' as const, label: t('request.bodyRaw') },
  ]
})

const formDataTypeOptions = computed(() => [
  { value: 'string', label: '[string]' },
  { value: 'file', label: '[file]' },
])

const splitContainerRef = ref<HTMLElement | null>(null)
const {
  size: responseHeight,
  dragging: responseDragging,
  startDrag: startResponseDrag,
  clampToContainer: clampResponseHeight,
  reloadFromStorage: reloadResponseHeight,
} = useContainerBoundedResize(splitContainerRef, {
  axis: 'y',
  min: 120,
  initial: 280,
  maxRatio: 0.75,
  // Per-interface height; legacy global key used only as first-time fallback.
  storageKey: () =>
    props.interfaceId != null
      ? `interface_response_height:${props.interfaceId}`
      : 'interface_response_height:create',
  fallbackStorageKey: 'interface_response_height',
})
const {
  size: grpcListWidth,
  dragging: grpcListDragging,
  startDrag: startGrpcListDrag,
} = useContainerBoundedResize(splitContainerRef, {
  axis: 'x',
  min: 180,
  initial: 240,
  maxRatio: 0.42,
  storageKey: 'grpc_method_list_width',
})

watch(
  () => props.interfaceId,
  () => {
    reloadResponseHeight()
  },
)

const folders = ref<FolderOption[]>([])
const saveUi = ref<'idle' | 'saving' | 'saved'>('idle')
const sending = ref(false)
const wsConnected = ref(false)
const wsConnecting = ref(false)
const wsDisconnecting = ref(false)
const wsSending = ref(false)
const wsSessionId = ref<string | null>(null)
const wsMessages = ref<WsMessageItem[]>([])
const wsResponseHeaders = ref<HeaderPair[]>([])
const wsStatusText = ref('')
const wsConnectError = ref(false)
const wsMessageListRef = ref<HTMLElement | null>(null)
let wsEventUnlisten: UnlistenFn | null = null
let wsMsgSeq = 0
let saveUiTimer: ReturnType<typeof setTimeout> | null = null
const protocol = ref<Protocol>('HTTP')
const method = ref('POST')
const isCreateMode = computed(() => props.mode === 'create')
const isProtocolOnly = computed(() => isProtocolOnlySelect(protocol.value))
const protocolOptions = computed(() =>
  switchableProtocols(protocol.value, isCreateMode.value).map((p) => ({
    value: p,
    label: protocolLabel(p),
  })),
)
const methodOptions = computed(() => methodsForProtocol(protocol.value))

function onProtocolChange(value: string | number | null) {
  const next = normalizeProtocol(value)
  if (next === protocol.value) return
  if (!isCreateMode.value) {
    const allowed = switchableProtocols(protocol.value, false)
    if (!allowed.includes(next)) return
  }
  protocol.value = next
}

// Keep method valid when protocol changes (picker or URL scheme).
watch(protocol, (next) => {
  const allowed = methodsForProtocol(next)
  if (!allowed.includes(method.value)) {
    method.value = allowed[0] ?? 'GET'
  }
  if (protocolFamily(next) !== 'ws' && (wsConnected.value || wsSessionId.value)) {
    void disconnectWsSession(true)
  }
  if (protocolFamily(next) === 'grpc') {
    if (requestTab.value !== 'message' && requestTab.value !== 'metadata') {
      requestTab.value = 'message'
    }
    bodyFormat.value = 'raw'
    if (!grpcServices.value.length) {
      applyGrpcMetadata([])
      if (!bodyRaw.value.trim()) bodyRaw.value = '{}'
    }
    return
  }
  if (requestTab.value === 'message' || requestTab.value === 'metadata') {
    requestTab.value = 'body'
  }
  if (protocolFamily(next) === 'ws') {
    if (requestTab.value === 'authorization') requestTab.value = 'body'
    if (bodyFormat.value !== 'raw') bodyFormat.value = 'raw'
  }
})

const urlBase = ref('')
/** Raw URL bar text. Kept as typed while editing; rebuilt only when Params change. */
const urlBarText = ref('')
/** True while parseUrlInput is applying bar → params (skip params → bar rebuild). */
let syncingUrlFromBar = false
const name = ref('')
const folderId = ref<string | number | null>(null)
const requestTab = ref<RequestTabId>('body')
watch(requestTab, () => {
  if (editorReady.value && trackDirty.value) emit('state-change')
})
const bodyFormat = ref<BodyFormat>('none')
const bodyRaw = ref('')
const rawBodyEditorRef = ref<{ formatContent: () => boolean } | null>(null)
const protoFileRef = ref<HTMLInputElement | null>(null)
const grpcProtoName = ref('')
const grpcProtoSource = ref('')
const grpcPackage = ref('')
const grpcServices = ref<GrpcService[]>([])
const grpcSelectedRpc = ref('')
const grpcRpcOptions = computed(() =>
  grpcServices.value.flatMap((svc) =>
    svc.methods.map((m) => ({
      value: m.rpc_path,
      label: m.rpc_path,
    })),
  ),
)
const validationDialogOpen = ref(false)
const validationDialogTitle = ref('')
const validationDialogMessage = ref('')

function showValidation(message: string, title?: string) {
  validationDialogTitle.value = title ?? t('create.validationTitle')
  validationDialogMessage.value = message
  validationDialogOpen.value = true
}
const formDataFields = ref<FormDataRow[]>([emptyParamRow()])
const workspaceDictionaries = ref<DictionaryEntryRow[]>([])
let workspaceDictionariesWorkspaceId: number | null = null

async function refreshWorkspaceDictionaries(force = false) {
  const wsId = props.workspaceId
  if (!wsId) {
    workspaceDictionaries.value = []
    workspaceDictionariesWorkspaceId = null
    return
  }
  if (!force && workspaceDictionariesWorkspaceId === wsId) {
    return
  }
  try {
    workspaceDictionaries.value = await invoke<DictionaryEntryRow[]>('list_workspace_dictionaries', {
      workspaceId: wsId,
    })
    workspaceDictionariesWorkspaceId = wsId
  } catch {
    workspaceDictionaries.value = []
    workspaceDictionariesWorkspaceId = null
  }
}

function formDataRowDictMatched(name: string) {
  return paramHasDictMatch(workspaceDictionaries.value, name)
}

function applyDictionaryToFormDataRow(idx: number, entry: DictionaryEntryRow) {
  const row = formDataFields.value[idx]
  if (!row) return
  row.name = entry.name
  row.description = entry.description ?? ''
  // Keep type as-is. Only fill value when empty (text rows only).
  if (row.type !== 'file' && !row.example.trim()) {
    const fallback = (entry.default_value ?? '').trim()
    if (fallback) row.example = fallback
  }
  onFormDataChange(idx)
  emit('state-change')
}

function applyDictionaryDescriptionsToAllFormData() {
  const count = applyDictDescriptionsToFormData(formDataFields.value, workspaceDictionaries.value)
  if (count > 0) {
    normalizeFormDataFields()
    emit('state-change')
    showToast(t('dict.applyAllFormDataSuccess', { count }), 1500, 'success')
  } else {
    showToast(t('dict.applyAllFormDataNone'), 1600, 'warning')
  }
}

watch(
  () => [props.workspaceId, bodyFormat.value] as const,
  ([, format]) => {
    if (format === 'form-data') void refreshWorkspaceDictionaries(true)
  },
  { immediate: true },
)
const importJsonDialogOpen = ref(false)
const paramTableEl = ref<HTMLElement | null>(null)
const {
  widths: tableColWidths,
  start: startTableColResize,
  fitToWidth: fitTableColWidths,
} = useColumnResize(
  'request_table_col_widths',
  { key: 300, type: 90, value: 400, desc: 400 }, // default
  { key: 150, type: 90, value: 200, desc: 80 },  // min
  { key: 500, type: 140, value: 700, desc: 500 }, // max
)

const paramGridColumns = computed(
  () =>
    `40px ${tableColWidths.value.key}px ${tableColWidths.value.value}px ${tableColWidths.value.desc}px 40px`,
)
const formDataGridColumns = computed(
  () =>
    `40px ${tableColWidths.value.key}px ${tableColWidths.value.type}px ${tableColWidths.value.value}px ${tableColWidths.value.desc}px 72px`,
)

function currentParamTableLayout(): 'params' | 'form-data' {
  return requestTab.value === 'body' && bodyFormat.value === 'form-data' ? 'form-data' : 'params'
}

function fitActiveParamTable() {
  const el = paramTableEl.value
  if (!el) return
  fitTableColWidths(el.clientWidth, currentParamTableLayout())
}

let paramTableRo: ResizeObserver | null = null

watch(
  paramTableEl,
  (el) => {
    if (paramTableRo) {
      paramTableRo.disconnect()
      paramTableRo = null
    }
    if (!el || typeof ResizeObserver === 'undefined') return
    paramTableRo = new ResizeObserver(() => fitActiveParamTable())
    paramTableRo.observe(el)
    fitActiveParamTable()
  },
)

const requestHeaders = ref<HeaderRow[]>([defaultHeaderRow()])
const showHiddenHeaders = ref(false)
const disabledAutoHeaders = ref<Set<string>>(new Set())
const queryParams = ref<QueryParamRow[]>([defaultQueryRow()])

function selectAllState(rows: { enabled?: boolean }[]) {
  const total = rows.length
  const enabled = rows.filter((r) => r.enabled !== false).length
  return {
    checked: total > 0 && enabled === total,
    indeterminate: enabled > 0 && enabled < total,
  }
}

const queryParamsSelectAll = computed(() => selectAllState(queryParams.value))
const requestHeadersSelectAll = computed(() => selectAllState(requestHeaders.value))
const formDataSelectAll = computed(() => selectAllState(formDataFields.value))

function setCheckboxIndeterminate(el: unknown, indeterminate: boolean) {
  if (el instanceof HTMLInputElement) el.indeterminate = indeterminate
}

function setAllRowsEnabled(rows: { enabled: boolean }[], enabled: boolean) {
  for (const row of rows) row.enabled = enabled
}
const responseHeaders = ref<ParamRow[]>([emptyRow()])
const resultName = ref('')
const resultStatus = ref('200')
const resultSchema = ref('')
const exampleName = ref('')
const exampleStatus = ref('200')
const exampleBody = ref('')

const authType = ref<'none' | 'bearer' | 'basic'>('none')
const authBearerToken = ref('')
const authBasicUser = ref('')
const authBasicPassword = ref('')

const authTypeHint = computed(() => {
  if (authType.value === 'bearer') return t('request.authBearerHint')
  if (authType.value === 'basic') return t('request.authBasicHint')
  return t('request.authNoneHint')
})

function autoHeaderContext() {
  return {
    url: applyEnvVars(displayUrl.value.trim(), envVarMap.value),
    bodyFormat: bodyFormat.value,
    rawBody: bodyFormat.value === 'raw' ? stripJsonComments(bodyRaw.value) : bodyRaw.value,
    authType: authType.value,
    authBearerToken: authBearerToken.value,
    authBasicUser: authBasicUser.value,
    authBasicPassword: authBasicPassword.value,
    userHeaders: requestHeaders.value.map((h) => ({
      key: h.name,
      enabled: h.enabled,
    })),
    disabledAutoHeaders: disabledAutoHeaders.value,
  }
}

function autoHeaderEnabled(key: string) {
  return isAutoHeaderEnabled(autoHeaderContext(), key)
}

function setAutoHeaderEnabled(key: string, enabled: boolean) {
  const normalized = key.trim().toLowerCase()
  if (!normalized) return
  const next = new Set(disabledAutoHeaders.value)
  if (enabled) next.delete(normalized)
  else next.add(normalized)
  disabledAutoHeaders.value = next
}

const hiddenHeaders = computed(() => (isGrpc.value ? [] : buildAutoHeaders(autoHeaderContext())))

const headerTabCount = computed(() => {
  const userCount = requestHeaders.value.filter((r) => r.name.trim() && r.enabled !== false).length
  const autoCount = hiddenHeaders.value.filter((h) => autoHeaderEnabled(h.key)).length
  return userCount + autoCount
})

type ResponseBodyType = 'JSON' | 'Text'

const response = ref<SendResponse | null>(null)
const responseTab = ref<'body' | 'headers'>('body')
const responseBodyType = ref<ResponseBodyType>('JSON')
const responseScrollTop = ref(0)
const responseBodyEditorRef = ref<{
  getScrollTop: () => number
  setScrollTop: (top: number) => void
} | null>(null)
const responseBodyTypes = computed(() => [
  { value: 'JSON' as const, label: t('response.bodyJson') },
  { value: 'Text' as const, label: t('response.bodyText') },
])

const responseDisplayBody = computed(() => {
  const raw = response.value?.body ?? ''
  if (responseBodyType.value !== 'JSON') return raw
  return formatJsonLenient(raw.trim()) ?? raw
})

function onResponseBodyScroll(top: number) {
  responseScrollTop.value = top
}

function restoreResponseBodyScroll(top = responseScrollTop.value) {
  const next = Math.max(0, Number(top) || 0)
  responseScrollTop.value = next
  void nextTick(() => {
    void nextTick(() => {
      responseBodyEditorRef.value?.setScrollTop(next)
    })
  })
}

watch(responseTab, (tab) => {
  if (tab === 'body' && response.value && !isSendError.value) {
    restoreResponseBodyScroll()
  }
})

watch(responseDisplayBody, () => {
  // Doc rewrite resets CM scroll; re-apply after content settles when we have a saved offset.
  if (responseTab.value === 'body' && responseScrollTop.value > 0) {
    restoreResponseBodyScroll()
  }
})

const responseBodySize = computed(() => {
  const bytes = new TextEncoder().encode(response.value?.body ?? '').length
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
})

function detectResponseBodyType(body: string): ResponseBodyType {
  const trimmed = body.trim()
  if (!trimmed) return 'Text'
  if (
    (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
    (trimmed.startsWith('[') && trimmed.endsWith(']'))
  ) {
    if (formatJsonLenient(trimmed)) return 'JSON'
  }
  return 'Text'
}

const isSendError = computed(() => Boolean(response.value?.error) || response.value?.status === 0)

const responseStatusClass = computed(() => {
  const s = response.value?.status ?? 0
  if (response.value?.error || s === 0) return 'err'
  if (s >= 200 && s < 300) return 'ok'
  if (s >= 400) return 'err'
  return 'warn'
})

function invokeErrorMessage(e: unknown): string {
  if (typeof e === 'string' && e.trim()) return e.trim()
  if (e instanceof Error && e.message.trim()) return e.message.trim()
  if (e && typeof e === 'object' && 'message' in e) {
    const msg = String((e as { message: unknown }).message ?? '').trim()
    if (msg) return msg
  }
  const fallback = String(e ?? '').trim()
  return fallback && fallback !== '[object Object]' ? fallback : 'request failed'
}

function applySendError(e: unknown, durationMs = 0) {
  responseScrollTop.value = 0
  response.value = {
    status: 0,
    status_text: 'Error',
    headers: [],
    body: invokeErrorMessage(e),
    duration_ms: durationMs,
    history_id: 0,
    error: true,
  }
  responseBodyType.value = 'Text'
  responseTab.value = 'body'
}

function emptyRow(): ParamRow {
  return { name: '', type: 'string', required: false, description: '', example: '' }
}

function defaultHeaderRow(): HeaderRow {
  return emptyParamRow()
}

function emptyParamRow(): QueryParamRow {
  return { ...emptyRow(), enabled: true }
}

function isLegacyDefaultJsonContentType(rows: HeaderRow[]) {
  const named = rows.filter((r) => r.name.trim())
  if (named.length !== 1) return false
  const row = named[0]
  return (
    row.name.trim().toLowerCase() === 'content-type' &&
    row.example.trim().toLowerCase() === 'application/json' &&
    !row.description.trim()
  )
}

function defaultQueryRow(): QueryParamRow {
  return emptyParamRow()
}

function isParamRowEmpty(row: Pick<QueryParamRow, 'name' | 'example' | 'description'>) {
  return !row.name.trim() && !row.example.trim() && !row.description.trim()
}

function normalizeQueryParams() {
  const rows = queryParams.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function normalizeRequestHeaders() {
  const rows = requestHeaders.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function normalizeFormDataFields() {
  const rows = formDataFields.value
  while (
    rows.length > 1 &&
    isParamRowEmpty(rows[rows.length - 1]) &&
    isParamRowEmpty(rows[rows.length - 2])
  ) {
    rows.pop()
  }
  if (!rows.length || !isParamRowEmpty(rows[rows.length - 1])) {
    rows.push(emptyParamRow())
  }
}

function onImportFormDataJson(payload: { mode: FormDataImportMode; jsonText: string }) {
  try {
    const imported = applyFormDataJsonImport(formDataFields.value, payload.jsonText, payload.mode)
    formDataFields.value = imported.map((row) => ({
      name: row.name,
      type: row.type,
      example: row.example,
      description: row.description,
      required: row.required,
      enabled: row.enabled,
      fileData: row.type === 'file' ? row.fileData : undefined,
      fileName: row.type === 'file' ? row.fileName : undefined,
    }))
    normalizeFormDataFields()
    importJsonDialogOpen.value = false
    showToast(t('request.importJsonSuccess'), 1500, 'success')
  } catch (error) {
    if (error instanceof Error) {
      if (error.message === 'empty') {
        showToast(t('request.importJsonEmpty'), 1600, 'warning')
        return
      }
      if (error.message === 'root-object') {
        showToast(t('request.importJsonRootInvalid'), 1600, 'warning')
        return
      }
    }
    showToast(t('request.importJsonInvalid'), 1600, 'warning')
  }
}

function onFormDataChange(idx: number) {
  if (idx !== formDataFields.value.length - 1) return
  if (isParamRowEmpty(formDataFields.value[idx])) return
  normalizeFormDataFields()
}

const FORM_DATA_VALUE_MAX_HEIGHT = 240

function formDataValueShell(el: HTMLTextAreaElement): HTMLElement | null {
  return el.closest('.param-table__value-shell') as HTMLElement | null
}

function formDataValueMaxWidthPx(el: HTMLTextAreaElement): number {
  const cell = el.closest('.param-table__value') as HTMLElement | null
  const table = el.closest('.param-table') as HTMLElement | null
  if (!cell || !table) return Math.max(el.offsetWidth, 160)
  const cellRect = cell.getBoundingClientRect()
  const tableRect = table.getBoundingClientRect()
  // Allow growing past the Value column up to the table's right edge.
  return Math.max(Math.floor(tableRect.right - cellRect.left - 16), Math.floor(cellRect.width - 12), 160)
}

function resizeFormDataValueInput(el: HTMLTextAreaElement, expanded: boolean) {
  const shell = formDataValueShell(el)
  if (!shell) return
  if (!expanded) {
    shell.style.height = ''
    shell.style.width = ''
    shell.style.maxWidth = ''
    el.style.height = ''
    el.style.overflowY = ''
    return
  }
  shell.style.maxWidth = `${formDataValueMaxWidthPx(el)}px`
  el.style.overflowY = 'hidden'
  el.style.height = '0px'
  const needed = el.scrollHeight + 2
  const next = Math.min(Math.max(needed, 34), FORM_DATA_VALUE_MAX_HEIGHT)
  shell.style.height = `${next}px`
  el.style.height = '100%'
  el.style.overflowY = needed > FORM_DATA_VALUE_MAX_HEIGHT ? 'auto' : 'hidden'
}

function onFormDataValueFocus(event: FocusEvent) {
  const el = event.target as HTMLTextAreaElement
  requestAnimationFrame(() => {
    if (document.activeElement === el) resizeFormDataValueInput(el, true)
  })
}

function onFormDataValueBlur(event: FocusEvent) {
  const el = event.target as HTMLTextAreaElement
  resizeFormDataValueInput(el, false)
}

function onFormDataValueInput(idx: number, event: Event) {
  onFormDataChange(idx)
  const el = event.target as HTMLTextAreaElement
  if (document.activeElement === el) {
    resizeFormDataValueInput(el, true)
  }
}

function onFormDataTypeChange(idx: number) {
  const row = formDataFields.value[idx]
  if (row.type === 'file') {
    row.example = ''
    row.fileData = undefined
    row.fileName = undefined
  } else {
    row.fileData = undefined
    row.fileName = undefined
  }
  onFormDataChange(idx)
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = String(reader.result ?? '')
      const comma = result.indexOf(',')
      resolve(comma >= 0 ? result.slice(comma + 1) : result)
    }
    reader.onerror = () => reject(reader.error ?? new Error('failed to read file'))
    reader.readAsDataURL(file)
  })
}

async function onFormDataFileChange(idx: number, event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  const row = formDataFields.value[idx]
  if (!file) {
    row.example = ''
    row.fileName = undefined
    row.fileData = undefined
    onFormDataChange(idx)
    return
  }
  try {
    row.example = file.name
    row.fileName = file.name
    row.fileData = await readFileAsBase64(file)
  } catch {
    row.example = ''
    row.fileName = undefined
    row.fileData = undefined
    showToast(t('request.formDataFileMissing', { name: row.name.trim() || t('common.key') }), 1800, 'warning')
  }
  onFormDataChange(idx)
}

function removeFormDataRow(idx: number) {
  if (formDataFields.value.length <= 1) return
  formDataFields.value.splice(idx, 1)
  normalizeFormDataFields()
}

function hasFormDataContent(fields: FormDataRow[]) {
  return fields.some(
    (r) =>
      r.enabled !== false &&
      (r.name.trim() || r.example.trim() || r.description.trim()),
  )
}

function resolveBodyFormat(raw: string, fields: FormDataRow[]): BodyFormat {
  if (hasFormDataContent(fields)) return 'form-data'
  if (raw.trim()) return 'raw'
  return 'none'
}

function formatBodyRaw() {
  rawBodyEditorRef.value?.formatContent()
}

function autoGenerateGrpcMessage() {
  const method = findGrpcMethod(currentGrpcCatalog(), grpcSelectedRpc.value)
  const next = fillGrpcMessageSamples(bodyRaw.value, method?.message ?? '')
  if (!next) {
    showToast(t('grpc.autoGenerateEmpty'), 1600, 'warning')
    return
  }
  bodyRaw.value = next
}

function buildRequestBodyForSend(): string | null {
  if (bodyFormat.value === 'none') return null
  if (bodyFormat.value === 'raw') {
    const raw = bodyRaw.value
    if (!raw) return null
    return stripJsonComments(raw)
  }
  return null
}

function buildFormDataForSend(): FormDataSendField[] | null {
  if (bodyFormat.value !== 'form-data') return null
  const fields = formDataFields.value.filter((r) => r.enabled !== false && r.name.trim())
  if (!fields.length) return null

  const result: FormDataSendField[] = []
  for (const row of fields) {
    const name = applyEnvVars(row.name.trim(), envVarMap.value)
    if (row.type === 'file') {
      if (!row.fileData) {
        throw new Error(t('request.formDataFileMissing', { name }))
      }
      result.push({
        name,
        value: '',
        type: 'file',
        file_name: row.fileName || row.example || 'file',
        file_data: row.fileData,
      })
    } else {
      result.push({
        name,
        value: applyEnvVars(row.example, envVarMap.value),
        type: 'string',
      })
    }
  }
  return result.length ? result : null
}

function onQueryParamChange(idx: number) {
  if (idx !== queryParams.value.length - 1) return
  if (isParamRowEmpty(queryParams.value[idx])) return
  normalizeQueryParams()
}

function onHeaderChange(idx: number) {
  if (idx !== requestHeaders.value.length - 1) return
  if (isParamRowEmpty(requestHeaders.value[idx])) return
  normalizeRequestHeaders()
}

function removeQueryRow(idx: number) {
  if (queryParams.value.length <= 1) return
  queryParams.value.splice(idx, 1)
  normalizeQueryParams()
}

function removeHeaderRow(idx: number) {
  if (requestHeaders.value.length <= 1) return
  requestHeaders.value.splice(idx, 1)
  normalizeRequestHeaders()
}

/** Encode for URL bar display; keep `/` readable (do not show as %2F). */
function encodeQueryComponent(value: string) {
  return encodeURIComponent(value).replace(/%2F/gi, '/')
}

function decodeQueryComponent(value: string) {
  try {
    return decodeURIComponent(value.replace(/\+/g, ' '))
  } catch {
    // Incomplete sequences like "%2" must stay as-is; re-encoding "%2" → "%252"
    // would make Backspace appear to "put characters back".
    return value.replace(/\+/g, ' ')
  }
}

function buildQuerySuffix() {
  const params = queryParams.value.filter((r) => r.enabled !== false && r.name.trim())
  if (!params.length) return ''
  const qs = params
    .map((r) => `${encodeQueryComponent(r.name.trim())}=${encodeQueryComponent(r.example)}`)
    .join('&')
  return `?${qs}`
}

function syncUrlBarFromParams() {
  if (syncingUrlFromBar) return
  urlBarText.value = `${urlBase.value}${buildQuerySuffix()}`
}

watch(queryParams, syncUrlBarFromParams, { deep: true })

const displayUrl = computed({
  get() {
    return urlBarText.value
  },
  set(val: string) {
    parseUrlInput(val)
  },
})

const highlightedUrlHtml = computed(() => {
  const raw = displayUrl.value
  if (!raw) return ''
  let html = ''
  let last = 0
  ENV_VAR_RE.lastIndex = 0
  let match: RegExpExecArray | null
  while ((match = ENV_VAR_RE.exec(raw))) {
    html += escapeHtml(raw.slice(last, match.index))
    const full = match[0]
    const key = match[1].trim()
    const ok = envVarKeys.value.has(key)
    const end = match.index + full.length
    html += `<span class="url-var ${ok ? 'url-var--ok' : 'url-var--missing'}">${escapeHtml(full)}</span>`
    last = end
  }
  html += escapeHtml(raw.slice(last))
  return html || '&nbsp;'
})

watch(displayUrl, () => {
  hideUrlVarTip()
  urlVarHoverKey = null
  urlVarHitBoxesFor = ''
})

function clearQueryParams() {
  queryParams.value = [emptyParamRow()]
  normalizeQueryParams()
}

function applyParsedCurl(parsed: ParsedCurl) {
  method.value = (parsed.method || 'GET').toUpperCase()
  parseUrlInput(parsed.url || '')

  if (parsed.headers.length) {
    requestHeaders.value = parsed.headers.map((h) => ({
      ...emptyRow(),
      name: h.key,
      example: h.value,
      enabled: true,
    }))
  } else {
    requestHeaders.value = [emptyParamRow()]
  }
  normalizeRequestHeaders()

  // Prefer explicit Authorization header from curl over auth pane to avoid duplicates.
  authType.value = 'none'
  authBearerToken.value = ''
  authBasicUser.value = ''
  authBasicPassword.value = ''

  if (parsed.formData?.length) {
    bodyFormat.value = 'form-data'
    bodyRaw.value = ''
    formDataFields.value = parsed.formData.map((f) => ({
      ...emptyRow(),
      name: f.name,
      type: f.type,
      example: f.value,
      enabled: true,
    }))
    normalizeFormDataFields()
    requestTab.value = 'body'
  } else if (parsed.body != null && parsed.body !== '') {
    bodyFormat.value = 'raw'
    bodyRaw.value = parsed.body
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    requestTab.value = 'body'
  } else {
    bodyFormat.value = 'none'
    bodyRaw.value = ''
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    requestTab.value = parsed.headers.length ? 'headers' : 'params'
  }

  showToast(t('request.curlImported'), 1800, 'success')
}

function onUrlPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  if (looksLikeCurl(text)) {
    const parsed = parseCurl(text)
    if (!parsed) return
    e.preventDefault()
    urlUndo.checkpoint(urlSnapshot())
    applyParsedCurl(parsed)
    void nextTick(() => urlUndo.commit(urlSnapshot()))
    return
  }

  // Copied URLs often start with a leading CR/LF; keep it out of the input
  // so the transparent caret overlay does not drift (looks like a trailing space).
  const trimmed = text.replace(/^\uFEFF/, '').trim()
  if (trimmed === text) return
  e.preventDefault()
  urlUndo.checkpoint(urlSnapshot())
  const el = urlInputRef.value
  const cur = displayUrl.value
  const start = el?.selectionStart ?? cur.length
  const end = el?.selectionEnd ?? start
  parseUrlInput(cur.slice(0, start) + trimmed + cur.slice(end))
  void nextTick(() => {
    if (el) {
      const pos = Math.min(start + trimmed.length, el.value.length)
      try {
        el.setSelectionRange(pos, pos)
      } catch {
        /* ignore */
      }
    }
    urlUndo.commit(urlSnapshot())
  })
}

function parseUrlInput(raw: string) {
  // Strip BOM / leading/trailing whitespace (CR/LF/space). A leading newline
  // survives in <input> value and breaks caret alignment vs the highlight layer.
  raw = raw.replace(/^\uFEFF/, '').trim()
  // Keep the bar text exactly as typed (after trim). Do not rebuild from decoded
  // params on each keystroke — that rewrites incomplete escapes (e.g. "%2"→"%252")
  // and drops a trailing "?" before any key is entered.
  syncingUrlFromBar = true
  try {
    if (!raw) {
      urlBase.value = ''
      clearQueryParams()
      return
    }

    // Keep scheme in the URL bar; only update protocol metadata.
    // After create, only allow same-family protocol changes.
    const setProtocolFromUrl = (next: Protocol) => {
      if (isCreateMode.value || switchableProtocols(protocol.value, false).includes(next)) {
        protocol.value = next
      }
    }
    if (/^https:\/\//i.test(raw)) {
      setProtocolFromUrl('HTTPS')
    } else if (/^http:\/\//i.test(raw)) {
      setProtocolFromUrl('HTTP')
    } else if (/^wss:\/\//i.test(raw)) {
      setProtocolFromUrl('WS')
    } else if (/^ws:\/\//i.test(raw)) {
      setProtocolFromUrl('WS')
    } else if (/^tcp:\/\//i.test(raw)) {
      setProtocolFromUrl('TCP')
    } else if (/^udp:\/\//i.test(raw)) {
      setProtocolFromUrl('UDP')
    }

    const hashIndex = raw.indexOf('#')
    const withoutHash = hashIndex >= 0 ? raw.slice(0, hashIndex) : raw
    const hash = hashIndex >= 0 ? raw.slice(hashIndex) : ''
    const qIndex = withoutHash.indexOf('?')

    if (qIndex === -1) {
      // Keep exactly what was typed (e.g. "http://"); never force "/".
      urlBase.value = `${withoutHash}${hash}`
      clearQueryParams()
      return
    }

    urlBase.value = `${withoutHash.slice(0, qIndex)}${hash}`
    const search = withoutHash.slice(qIndex + 1)
    if (!search) {
      clearQueryParams()
      return
    }

    const rows: QueryParamRow[] = []
    for (const part of search.split('&')) {
      if (!part) continue
      const eq = part.indexOf('=')
      const key = eq >= 0 ? part.slice(0, eq) : part
      const value = eq >= 0 ? part.slice(eq + 1) : ''
      rows.push({
        ...emptyRow(),
        name: decodeQueryComponent(key),
        example: decodeQueryComponent(value),
        enabled: true,
      })
    }
    queryParams.value = rows
    normalizeQueryParams()
  } finally {
    urlBarText.value = raw
    syncingUrlFromBar = false
  }
}

/** Persist URL as typed (including scheme/host). Never force a leading "/". */
function urlForSave(): string {
  // Previously stripped `https://host` down to path-only; host-only URLs like
  // `https://bucket.oss-accelerate.aliyuncs.com` became "" and vanished on reopen.
  return displayUrl.value.trim()
}

function resolveRequestUrl(rawInput?: string): string | null {
  const raw = (rawInput ?? displayUrl.value).trim()
  if (!raw) return null
  // Absolute URLs keep their own scheme (http(s) / ws(s) / …).
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(raw)) {
    try {
      return new URL(raw).href
    } catch {
      return null
    }
  }
  if (raw.startsWith('/')) {
    return null
  }
  const hostPath = raw.replace(/^\/*/, '')
  if (!hostPath) return null
  const schemeMap: Record<string, string> = {
    HTTP: 'http',
    HTTPS: 'https',
    WS: 'ws',
    TCP: 'tcp',
    UDP: 'udp',
    GRPC: 'grpc',
  }
  const scheme = schemeMap[protocol.value] ?? 'http'
  try {
    return new URL(`${scheme}://${hostPath}`).href
  } catch {
    return null
  }
}

/** WS send needs ws/wss. Map http(s) typed in the bar to the websocket scheme. */
function toWebSocketUrl(url: string): string {
  if (/^https:\/\//i.test(url)) return `wss://${url.slice(url.indexOf('://') + 3)}`
  if (/^http:\/\//i.test(url)) return `ws://${url.slice(url.indexOf('://') + 3)}`
  return url
}

const wsCanSend = computed(() => {
  if (bodyFormat.value === 'none') return false
  if (bodyFormat.value === 'raw') return Boolean(bodyRaw.value.trim())
  return false
})

function appendWsMessage(partial: Omit<WsMessageItem, 'id' | 'at'> & { at?: number }) {
  wsMsgSeq += 1
  wsMessages.value.push({
    id: `m-${wsMsgSeq}`,
    at: partial.at ?? Date.now(),
    direction: partial.direction,
    kind: partial.kind,
    messageType: partial.messageType,
    data: partial.data,
  })
  void nextTick(() => {
    const el = wsMessageListRef.value
    if (el) el.scrollTop = el.scrollHeight
  })
}

function clearWsMessages() {
  wsMessages.value = []
}

function formatWsTime(at: number) {
  try {
    return new Date(at).toLocaleTimeString()
  } catch {
    return String(at)
  }
}

function wsDirectionLabel(direction: WsMessageItem['direction']) {
  if (direction === 'in') return t('response.wsIncoming')
  if (direction === 'out') return t('response.wsOutgoing')
  return t('response.wsSystem')
}

function resetWsUiState() {
  wsConnected.value = false
  wsConnecting.value = false
  wsDisconnecting.value = false
  wsSending.value = false
  wsSessionId.value = null
  wsStatusText.value = ''
}

async function ensureWsEventListener() {
  if (wsEventUnlisten) return
  wsEventUnlisten = await listen<WsSessionEventPayload>('ws-session-event', (event) => {
    const payload = event.payload
    if (!payload?.session_id) return
    if (wsSessionId.value && payload.session_id !== wsSessionId.value) return

    const direction =
      payload.direction === 'in' || payload.direction === 'out' || payload.direction === 'system'
        ? payload.direction
        : 'system'

    if (payload.kind === 'connected') {
      wsConnected.value = true
      wsConnecting.value = false
      wsConnectError.value = false
      appendWsMessage({
        direction: 'system',
        kind: payload.kind,
        messageType: payload.message_type || 'system',
        data: payload.data,
      })
      return
    }

    if (payload.kind === 'sent' || payload.kind === 'message') {
      appendWsMessage({
        direction,
        kind: payload.kind,
        messageType: payload.message_type || 'text',
        data: payload.data,
      })
      return
    }

    if (payload.kind === 'error') {
      wsConnectError.value = true
      appendWsMessage({
        direction: 'system',
        kind: payload.kind,
        messageType: 'error',
        data: payload.data,
      })
      return
    }

    if (payload.kind === 'closed') {
      appendWsMessage({
        direction: 'system',
        kind: payload.kind,
        messageType: payload.message_type || 'system',
        data: payload.data || t('request.wsDisconnected'),
      })
      wsConnected.value = false
      wsConnecting.value = false
      wsDisconnecting.value = false
      wsSending.value = false
      wsSessionId.value = null
      if (!wsStatusText.value.includes('101')) {
        wsStatusText.value = payload.data || t('request.wsDisconnected')
      }
    }
  })
}

async function disconnectWsSession(silent = false) {
  const sessionId = wsSessionId.value
  if (!sessionId) {
    resetWsUiState()
    return
  }
  wsDisconnecting.value = true
  try {
    await invoke('ws_disconnect', { args: { session_id: sessionId } })
  } catch (e) {
    if (!silent) {
      appendWsMessage({
        direction: 'system',
        kind: 'error',
        messageType: 'error',
        data: invokeErrorMessage(e),
      })
    }
  } finally {
    // Closed event will clear session; fall back if it never arrives.
    window.setTimeout(() => {
      if (wsSessionId.value === sessionId) resetWsUiState()
    }, 800)
  }
}

async function onWsConnectToggle() {
  if (wsConnected.value || wsSessionId.value) {
    await disconnectWsSession()
    return
  }
  await refreshEnvVars()
  const substituted = applyEnvVars(displayUrl.value.trim(), envVarMap.value)
  const url = resolveRequestUrl(substituted)
  if (!url) {
    showToast(t('request.invalidUrl'), 1500, 'warning')
    return
  }
  await ensureWsEventListener()
  wsConnecting.value = true
  wsConnectError.value = false
  wsStatusText.value = ''
  responseTab.value = 'body'
  try {
    const result = await invoke<WsConnectResult>('ws_connect', {
      args: {
        url: toWebSocketUrl(url),
        headers: buildSendHeaders(),
        variables: Object.keys(envVarMap.value).length ? envVarMap.value : null,
      },
    })
    wsSessionId.value = result.session_id
    wsResponseHeaders.value = result.headers ?? []
    wsStatusText.value = result.status
      ? `${result.status} ${result.status_text}`
      : result.status_text
    wsConnected.value = true
    wsConnectError.value = false
    // Connected system line also comes from backend event; keep UI responsive immediately.
  } catch (e) {
    wsConnectError.value = true
    wsConnected.value = false
    wsSessionId.value = null
    const msg = invokeErrorMessage(e)
    wsStatusText.value = msg
    appendWsMessage({
      direction: 'system',
      kind: 'error',
      messageType: 'error',
      data: msg,
    })
  } finally {
    wsConnecting.value = false
    emit('state-change')
  }
}

async function onWsSend() {
  if (!wsConnected.value || !wsSessionId.value) {
    showToast(t('request.wsNeedConnect'), 1500, 'warning')
    return
  }
  const body = bodyFormat.value === 'raw' ? bodyRaw.value : ''
  if (!body.trim()) {
    showToast(t('request.wsEmptyBody'), 1500, 'warning')
    return
  }
  await refreshEnvVars()
  wsSending.value = true
  responseTab.value = 'body'
  try {
    await invoke('ws_send', {
      args: {
        session_id: wsSessionId.value,
        body,
        variables: Object.keys(envVarMap.value).length ? envVarMap.value : null,
      },
    })
  } catch (e) {
    appendWsMessage({
      direction: 'system',
      kind: 'error',
      messageType: 'error',
      data: invokeErrorMessage(e),
    })
  } finally {
    wsSending.value = false
  }
}

function compactParams(rows: Array<ParamRow & { enabled?: boolean }>) {
  return rows
    .filter((r) => r.name.trim())
    .map((r) => ({
      name: r.name.trim(),
      type: r.type,
      required: r.required,
      description: r.description,
      example: r.example,
      enabled: r.enabled !== false,
    }))
}

/** Keep selected file payload so tab switch / save / reopen can still send it. */
function compactFormDataFields(rows: FormDataRow[]) {
  return rows
    .filter((r) => r.name.trim())
    .map((r) => {
      const base = {
        name: r.name.trim(),
        type: r.type,
        required: r.required,
        description: r.description,
        example: r.example,
        enabled: r.enabled !== false,
      }
      if (r.type !== 'file') return base
      return {
        ...base,
        file_name: r.fileName || r.example || undefined,
        file_data: r.fileData || undefined,
      }
    })
}

function currentGrpcCatalog(): GrpcCatalog {
  return {
    proto_name: grpcProtoName.value,
    proto_source: grpcProtoSource.value,
    package: grpcPackage.value,
    selected_rpc: grpcSelectedRpc.value,
    services: grpcServices.value,
  }
}

function headersToGrpcMetadata() {
  return compactGrpcMetadata(
    requestHeaders.value.map((r) => ({
      name: r.name,
      example: r.example,
      description: r.description,
      enabled: r.enabled !== false,
    })),
  )
}

function applyGrpcMetadata(rows: ReturnType<typeof normalizeGrpcMetadata>) {
  requestHeaders.value = rows.map((r) => ({
    name: r.name,
    type: 'string',
    required: false,
    description: r.description,
    example: r.example,
    enabled: r.enabled,
  }))
  if (!requestHeaders.value.length) {
    requestHeaders.value = [emptyParamRow()]
  }
  normalizeRequestHeaders()
}

function flushGrpcDraft() {
  const method = findGrpcMethod(currentGrpcCatalog(), grpcSelectedRpc.value)
  if (!method) return
  method.message = bodyRaw.value
  method.metadata = headersToGrpcMetadata()
}

function loadGrpcDraft(rpcPath: string) {
  const method = findGrpcMethod(currentGrpcCatalog(), rpcPath)
  bodyRaw.value = method?.message?.trim() ? method.message : '{}'
  applyGrpcMetadata(normalizeGrpcMetadata(method?.metadata ?? []))
  bodyFormat.value = 'raw'
}

function selectGrpcRpc(rpcPath: string) {
  if (!rpcPath || rpcPath === grpcSelectedRpc.value) return
  flushGrpcDraft()
  grpcSelectedRpc.value = rpcPath
  loadGrpcDraft(rpcPath)
}

function onGrpcRpcChange(value: string | number | null) {
  if (typeof value === 'string' && value) selectGrpcRpc(value)
}

function pickProtoFile() {
  const input = protoFileRef.value
  if (!input) return
  input.value = ''
  input.click()
}

async function onProtoFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  try {
    const source = await file.text()
    const parsed = await invoke<{
      proto_name: string
      package: string
      services: GrpcService[]
    }>('parse_proto_source', {
      args: { filename: file.name, source },
    })
    applyParsedProto(parsed.proto_name || file.name, source, parsed)
  } catch (e) {
    showValidation(String(e))
  }
}

function applyParsedProto(
  filename: string,
  source: string,
  parsed: { proto_name: string; package: string; services: GrpcService[] },
) {
  protocol.value = 'GRPC'
  method.value = 'GRPC'
  const prevFileStem = grpcProtoName.value.replace(/\.proto$/i, '')
  grpcProtoName.value = parsed.proto_name || filename
  grpcProtoSource.value = source
  grpcPackage.value = parsed.package || ''
  grpcServices.value = (parsed.services ?? []).map((svc) => ({
    name: svc.name,
    full_name: svc.full_name,
    methods: (svc.methods ?? []).map((m) => ({
      ...m,
      message: m.message || '{}',
      metadata: normalizeGrpcMetadata(m.metadata),
    })),
  }))
  const fallbackName = grpcProtoName.value.replace(/\.proto$/i, '')
  if (
    !name.value.trim() ||
    name.value === t('create.api') ||
    (prevFileStem && name.value === prevFileStem)
  ) {
    name.value = fallbackName || name.value
  }
  grpcSelectedRpc.value = firstRpcPath(currentGrpcCatalog())
  loadGrpcDraft(grpcSelectedRpc.value)
  requestTab.value = 'message'
  bodyFormat.value = 'raw'
  showToast(t('grpc.imported'), 1500, 'success')
}

function resetGrpcState() {
  grpcProtoName.value = ''
  grpcProtoSource.value = ''
  grpcPackage.value = ''
  grpcServices.value = []
  grpcSelectedRpc.value = ''
}

function buildDetail() {
  if (isGrpc.value) {
    flushGrpcDraft()
    return {
      protocol: 'GRPC' as const,
      grpc: {
        proto_name: grpcProtoName.value,
        proto_source: grpcProtoSource.value,
        package: grpcPackage.value,
        selected_rpc: grpcSelectedRpc.value,
        services: grpcServices.value.map((svc) => ({
          ...svc,
          methods: svc.methods.map((m) => ({
            ...m,
            metadata: compactGrpcMetadata(m.metadata),
          })),
        })),
      },
    }
  }
  return {
    protocol: protocol.value,
    request_headers: compactParams(requestHeaders.value),
    query_params: compactParams(queryParams.value),
    request_body: {
      format: bodyFormat.value,
      raw: bodyRaw.value,
      data_type: 'Object',
      fields: compactFormDataFields(formDataFields.value),
    },
    authorization: {
      type: authType.value,
      bearer_token: authBearerToken.value,
      basic_username: authBasicUser.value,
      basic_password: authBasicPassword.value,
    },
    response_headers: compactParams(responseHeaders.value),
    response_results: [
      {
        name: resultName.value || t('create.defaultSuccess'),
        status_code: Number(resultStatus.value) || 200,
        format: 'json',
        data_type: 'Object',
        schema_raw: resultSchema.value,
        fields: [],
      },
    ],
    response_examples: [
      {
        name: exampleName.value || t('create.successExample'),
        status_code: Number(exampleStatus.value) || 200,
        content_type: 'application/json',
        body: exampleBody.value,
      },
    ],
  }
}

function buildSendHeaders() {
  let headers = requestHeaders.value
    .filter((h) => h.enabled !== false && h.name.trim())
    .map((h) => ({
      key: h.name.trim(),
      value: h.example,
      enabled: true,
    }))

  if (bodyFormat.value === 'form-data') {
    headers = headers.filter((h) => {
      const key = h.key.toLowerCase()
      return key !== 'content-type' && key !== 'content-length' && key !== 'transfer-encoding'
    })
  }

  if (
    autoHeaderEnabled('Authorization') &&
    authType.value === 'bearer' &&
    authBearerToken.value.trim()
  ) {
    headers.push({
      key: 'Authorization',
      value: `Bearer ${authBearerToken.value.trim()}`,
      enabled: true,
    })
  } else if (
    autoHeaderEnabled('Authorization') &&
    authType.value === 'basic' &&
    (authBasicUser.value || authBasicPassword.value)
  ) {
    const token = btoa(`${authBasicUser.value}:${authBasicPassword.value}`)
    headers.push({
      key: 'Authorization',
      value: `Basic ${token}`,
      enabled: true,
    })
  }

  if (!isGrpc.value) {
    for (const h of sendableAutoHeaders(autoHeaderContext())) {
      if (!headers.some((x) => x.key.toLowerCase() === h.key.toLowerCase())) {
        headers.push({ key: h.key, value: h.value, enabled: true })
      }
    }
  }

  return headers
}

function applyInitial(resetUi = false) {
  showHiddenHeaders.value = false
  disabledAutoHeaders.value = new Set()
  const init = props.initial
  name.value = init?.name ?? ''
  const initUrl = init?.url ?? ''
  parseUrlInput(initUrl)
  resetUrlUndo()

  const detail = init?.detail
  const d = detail && typeof detail === 'object' ? (detail as Record<string, unknown>) : null
  if (d && isKnownProtocol(d.protocol)) {
    protocol.value = normalizeProtocol(d.protocol)
  } else if (String(init?.method ?? '').toUpperCase() === 'GRPC') {
    protocol.value = 'GRPC'
  }

  const allowed = methodsForProtocol(protocol.value)
  const rawMethod = (init?.method ?? allowed[0] ?? 'POST').toUpperCase()
  method.value = allowed.includes(rawMethod) ? rawMethod : (allowed[0] ?? 'POST')

  const fromInitFolder =
    init?.folder_id != null && Number.isFinite(Number(init.folder_id))
      ? Number(init.folder_id)
      : null
  const fromDetailFolder = d ? Number(d.folder_id) : NaN
  folderId.value =
    fromInitFolder ??
    (Number.isFinite(fromDetailFolder) ? fromDetailFolder : null) ??
    (props.mode === 'create' ? 0 : null)
  const restored = init?.lastResponse ?? null
  response.value = restored
    ? {
        ...restored,
        headers: (restored.headers ?? []).map((h) => ({ ...h })),
      }
    : null
  if (response.value) {
    responseBodyType.value = detectResponseBodyType(response.value.body ?? '')
  }
  responseScrollTop.value =
    typeof init?.responseScrollTop === 'number' && Number.isFinite(init.responseScrollTop)
      ? Math.max(0, init.responseScrollTop)
      : 0
  if (resetUi) {
    requestTab.value = requestTabForProtocol(init?.requestPaneTab, protocol.value)
  }

  if (d) {
    if (Array.isArray(d.request_headers) && d.request_headers.length) {
      const mapped = d.request_headers.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
        enabled: x.enabled !== false,
      }))
      requestHeaders.value = isLegacyDefaultJsonContentType(mapped)
        ? [emptyParamRow()]
        : mapped
    } else {
      requestHeaders.value = [defaultHeaderRow()]
    }
    normalizeRequestHeaders()
    if (Array.isArray(d.query_params) && d.query_params.length) {
      queryParams.value = d.query_params.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
        enabled: x.enabled !== false,
      }))
    } else if (!queryParams.value.length) {
      queryParams.value = [emptyParamRow()]
    }
    normalizeQueryParams()
    if (Array.isArray(d.response_headers) && d.response_headers.length) {
      responseHeaders.value = d.response_headers.map((x: any) => ({
        name: String(x.name ?? ''),
        type: String(x.type ?? 'string'),
        required: !!x.required,
        description: String(x.description ?? ''),
        example: String(x.example ?? ''),
      }))
    }
    const body = d.request_body as Record<string, unknown> | undefined
    if (body) {
      bodyRaw.value = String(body.raw ?? '')
      if (Array.isArray(body.fields) && body.fields.length) {
        formDataFields.value = (body.fields as any[]).map((x) => {
          const type = x.type === 'file' ? 'file' : 'string'
          const fileName =
            type === 'file'
              ? String(x.file_name ?? x.fileName ?? x.example ?? '').trim() || undefined
              : undefined
          const fileData =
            type === 'file'
              ? String(x.file_data ?? x.fileData ?? '').trim() || undefined
              : undefined
          return {
            name: String(x.name ?? ''),
            type,
            required: !!x.required,
            description: String(x.description ?? ''),
            example: String(x.example ?? fileName ?? ''),
            enabled: x.enabled !== false,
            fileName,
            fileData,
          }
        })
      } else {
        formDataFields.value = [emptyParamRow()]
      }
      normalizeFormDataFields()
      bodyFormat.value = resolveBodyFormat(bodyRaw.value, formDataFields.value)
    } else {
      bodyRaw.value = ''
      formDataFields.value = [emptyParamRow()]
      normalizeFormDataFields()
      bodyFormat.value = 'none'
    }
    const auth = d.authorization as Record<string, unknown> | undefined
    if (auth) {
      const type = String(auth.type ?? 'none')
      authType.value = type === 'bearer' || type === 'basic' ? type : 'none'
      authBearerToken.value = String(auth.bearer_token ?? '')
      authBasicUser.value = String(auth.basic_username ?? '')
      authBasicPassword.value = String(auth.basic_password ?? '')
    } else {
      authType.value = 'none'
      authBearerToken.value = ''
      authBasicUser.value = ''
      authBasicPassword.value = ''
    }
    const results = d.response_results as any[] | undefined
    if (results?.[0]) {
      resultName.value = String(results[0].name ?? '')
      resultStatus.value = String(results[0].status_code ?? '200')
      resultSchema.value = String(results[0].schema_raw ?? '')
    }
    const examples = d.response_examples as any[] | undefined
    if (examples?.[0]) {
      exampleName.value = String(examples[0].name ?? '')
      exampleStatus.value = String(examples[0].status_code ?? '200')
      exampleBody.value = String(examples[0].body ?? '')
    }
  } else {
    requestHeaders.value = [defaultHeaderRow()]
    normalizeRequestHeaders()
    queryParams.value = [emptyParamRow()]
    normalizeQueryParams()
    bodyFormat.value = 'none'
    bodyRaw.value = ''
    formDataFields.value = [emptyParamRow()]
    normalizeFormDataFields()
    authType.value = 'none'
    authBearerToken.value = ''
    authBasicUser.value = ''
    authBasicPassword.value = ''
    resultName.value = t('create.defaultSuccess')
    exampleName.value = t('create.successExample')
  }

  const catalog = parseGrpcCatalog(d)
  if (protocolFamily(protocol.value) === 'grpc') {
    if (catalog) {
      grpcProtoName.value = catalog.proto_name
      grpcProtoSource.value = catalog.proto_source
      grpcPackage.value = catalog.package
      grpcServices.value = catalog.services
      grpcSelectedRpc.value = catalog.selected_rpc || firstRpcPath(catalog)
      loadGrpcDraft(grpcSelectedRpc.value)
    } else {
      resetGrpcState()
      bodyRaw.value = '{}'
      applyGrpcMetadata([])
    }
    if (requestTab.value !== 'message' && requestTab.value !== 'metadata') {
      requestTab.value = 'message'
    }
    bodyFormat.value = 'raw'
  } else {
    resetGrpcState()
  }

  if (response.value && !isSendError.value && responseTab.value === 'body') {
    restoreResponseBodyScroll(responseScrollTop.value)
  }
}

async function loadFolders() {
  try {
    folders.value = await invoke('list_local_folders', {
      workspaceId: props.workspaceId,
      projectId: props.projectId,
    })
    if (folderId.value == null && props.mode === 'create') {
      folderId.value = 0
    }
  } catch {
    folders.value = []
  }
}

async function onSend() {
  await refreshEnvVars()
  const substituted = applyEnvVars(displayUrl.value.trim(), envVarMap.value)
  const url = resolveRequestUrl(substituted)
  if (!url) {
    showToast(t('request.invalidUrl'), 1500, 'warning')
    return
  }
  if (isGrpc.value) {
    flushGrpcDraft()
    if (!grpcProtoSource.value.trim()) {
      showToast(t('grpc.needProto'), 1600, 'warning')
      return
    }
    if (!grpcSelectedRpc.value) {
      showToast(t('grpc.needMethod'), 1600, 'warning')
      return
    }
    const grpcMethod = findGrpcMethod(currentGrpcCatalog(), grpcSelectedRpc.value)
    if (grpcMethod?.client_streaming || grpcMethod?.server_streaming) {
      showToast(t('grpc.streamingUnsupported'), 1800, 'warning')
      return
    }
    sending.value = true
    responseScrollTop.value = 0
    response.value = null
    responseTab.value = 'body'
    const grpcStarted = Date.now()
    try {
      const result = await invoke<SendResponse>('invoke_grpc', {
        args: {
          endpoint: url,
          rpc_path: grpcSelectedRpc.value,
          proto_source: grpcProtoSource.value,
          proto_name: grpcProtoName.value || null,
          message_json: stripJsonComments(bodyRaw.value),
          metadata: buildSendHeaders(),
          tls: false,
          interface_id: props.interfaceId ?? null,
          variables: Object.keys(envVarMap.value).length ? envVarMap.value : null,
        },
      })
      responseScrollTop.value = 0
      response.value = result
      responseBodyType.value = detectResponseBodyType(result.body ?? '')
      responseTab.value = 'body'
    } catch (e) {
      applySendError(e, Date.now() - grpcStarted)
    } finally {
      sending.value = false
      emit('state-change')
    }
    return
  }
  let formData: FormDataSendField[] | null = null
  if (bodyFormat.value === 'form-data') {
    try {
      formData = buildFormDataForSend()
    } catch (e) {
      showToast(invokeErrorMessage(e), 1800, 'warning')
      return
    }
  }
  sending.value = true
  responseScrollTop.value = 0
  response.value = null
  responseTab.value = 'body'
  const started = Date.now()
  try {
    const result = await invoke<SendResponse>('send_http_request', {
      args: {
        method: method.value,
        url,
        headers: buildSendHeaders(),
        body: buildRequestBodyForSend(),
        form_data: formData,
        interface_id: props.interfaceId ?? null,
        variables: Object.keys(envVarMap.value).length ? envVarMap.value : null,
      },
    })
    responseScrollTop.value = 0
    response.value = result
    responseBodyType.value = detectResponseBodyType(result.body ?? '')
    responseTab.value = 'body'
  } catch (e) {
    applySendError(e, Date.now() - started)
  } finally {
    sending.value = false
    emit('state-change')
  }
}

function exportTabState() {
  const liveScroll =
    responseTab.value === 'body'
      ? (responseBodyEditorRef.value?.getScrollTop() ?? responseScrollTop.value)
      : responseScrollTop.value
  responseScrollTop.value = liveScroll
  return {
    name: name.value,
    method: method.value,
    url: displayUrl.value,
    folderId: folderId.value,
    lastResponse: response.value,
    requestPaneTab: requestTab.value,
    responseScrollTop: liveScroll,
    detail: {
      ...buildDetail(),
      folder_id: folderId.value,
      protocol: protocol.value,
    },
  }
}

function snapshotFromState(state: ReturnType<typeof exportTabState>) {
  return JSON.stringify({
    name: state.name,
    method: state.method,
    url: state.url,
    detail: state.detail ?? null,
  })
}

const baselineSnapshot = ref('')

function getSnapshot() {
  return snapshotFromState(exportTabState())
}

function syncBaseline() {
  baselineSnapshot.value = getSnapshot()
}

function setBaseline(snapshot: string) {
  baselineSnapshot.value = snapshot
}

function isDirty() {
  if (!baselineSnapshot.value) return false
  return getSnapshot() !== baselineSnapshot.value
}

async function onSave(): Promise<boolean> {
  // Local folder ids are negative (next_local_id); 0 means project root.
  const folderIdNum =
    folderId.value == null || folderId.value === '' ? 0 : Number(folderId.value)
  if (!Number.isFinite(folderIdNum)) {
    showValidation(t('create.folderRequired'))
    return false
  }
  if (!name.value.trim()) {
    showValidation(t('create.nameRequired'))
    return false
  }
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
  saveUi.value = 'saving'
  try {
    // Keep exactly what was typed; never force a leading "/".
    // WS/UDP typically store host:port — prefixing "/" would break reopen/send.
    const saveUrl = urlForSave()

    let ok = false
    if (props.mode === 'create') {
      const row = await invoke<{ id: number }>('create_local_interface', {
        args: {
          workspace_id: props.workspaceId,
          project_id: props.projectId,
          folder_id: folderIdNum,
          name: name.value.trim(),
          method: method.value,
          url: saveUrl,
          status: props.initial?.status ?? 3,
          detail: buildDetail(),
        },
      })
      emit('saved', row.id)
      ok = true
    } else if (props.interfaceId != null) {
      await invoke('update_local_interface', {
        args: {
          id: props.interfaceId,
          workspace_id: props.workspaceId,
          project_id: props.projectId,
          folder_id: folderIdNum,
          name: name.value.trim(),
          method: method.value,
          url: saveUrl,
          status: props.initial?.status ?? 3,
          detail: buildDetail(),
        },
      })
      emit('saved', props.interfaceId)
      ok = true
    }
    if (ok) {
      saveUi.value = 'saved'
      saveUiTimer = setTimeout(() => {
        saveUi.value = 'idle'
        saveUiTimer = null
      }, 1400)
      return true
    }
    saveUi.value = 'idle'
    return false
  } catch (e) {
    saveUi.value = 'idle'
    alert(String(e))
    return false
  }
}

defineExpose({ save: onSave, exportTabState, getSnapshot, syncBaseline, setBaseline, isDirty })

const editorReady = ref(false)
const trackDirty = ref(false)
let editorAlive = false

const contentSnapshot = computed(() => getSnapshot())

onMounted(async () => {
  editorAlive = true
  baselineSnapshot.value = ''
  await Promise.all([loadFolders(), refreshEnvVars()])
  if (!editorAlive) return
  applyInitial(true)
  clampResponseHeight()
  await nextTick()
  if (!editorAlive) return
  emit('editor-ready')
  await nextTick()
  if (!editorAlive) return
  if (!baselineSnapshot.value) syncBaseline()
  trackDirty.value = true
  editorReady.value = true
})

onUnmounted(() => {
  editorAlive = false
  trackDirty.value = false
  editorReady.value = false
  baselineSnapshot.value = ''
  hideUrlVarTip()
  urlUndo.dispose()
  if (paramTableRo) {
    paramTableRo.disconnect()
    paramTableRo = null
  }
  if (saveUiTimer) {
    clearTimeout(saveUiTimer)
    saveUiTimer = null
  }
  void disconnectWsSession(true)
  if (wsEventUnlisten) {
    wsEventUnlisten()
    wsEventUnlisten = null
  }
})

watch(
  () => [props.workspaceId, props.projectId, props.initial],
  async (next, prev) => {
    if (!editorAlive) return
    trackDirty.value = false
    baselineSnapshot.value = ''
    const projectChanged =
      Array.isArray(prev) && (next[0] !== prev[0] || next[1] !== prev[1])
    // Project switch: wait for folders so the folder select stays consistent.
    // Same-project tab switch: apply immediately to avoid field flash.
    if (projectChanged) {
      await loadFolders()
      if (!editorAlive) return
    }
    applyInitial(true)
    await nextTick()
    if (!editorAlive) return
    emit('editor-ready')
    await nextTick()
    if (!editorAlive) return
    if (!baselineSnapshot.value) syncBaseline()
    trackDirty.value = true
    editorReady.value = true
  },
)

watch(
  contentSnapshot,
  () => {
    if (editorReady.value && trackDirty.value) emit('state-change')
  },
  { flush: 'post' },
)
</script>

<style scoped>
.body-pane {
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: hidden;
}

.body-pane > .param-table {
  flex: 1 1 auto;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
}

.body-pane > .param-table .param-table__row--head {
  position: sticky;
  top: 0;
  z-index: 4;
}

/* Keep resize handles stuck with the sticky form-data header while scrolling. */
.body-pane > .param-table :deep(.param-table__resize-overlay) {
  position: sticky;
  top: 0;
  z-index: 5;
  margin-bottom: calc(-1 * var(--param-table-head-height, 40px));
}

.body-format-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  min-height: 28px;
  position: relative;
  z-index: 2;
}

.body-format-row--grpc {
  padding-left: 10px;
}

.body-format-action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  margin-right: 10px;
  border: 0;
  background: transparent;
  appearance: none;
  color: var(--color-text-secondary);
  padding: 0;
  height: 24px;
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  transition: color 0.15s ease;
}

.body-format-action:hover {
  color: var(--color-interface-field-text);
}

.body-format-switch {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  height: 28px;
  padding: 2px;
  gap: 2px;
  box-sizing: border-box;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: 4px;
  background: color-mix(in srgb, var(--color-interface-bar-border) 28%, transparent);
}

.body-format-switch__btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0 14px 1px;
  height: 100%;
  border-radius: 2px;
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    box-shadow 0.15s ease;
}

.body-format-switch__btn:hover:not(.body-format-switch__btn--active) {
  color: var(--color-interface-field-text);
  background: color-mix(in srgb, var(--color-interface-accent) 8%, transparent);
}

.body-format-switch__btn--active {
  background: var(--color-workspace-content);
  color: var(--color-interface-field-text);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

.interface-response-type-switch {
  flex-shrink: 0;
  height: auto;
}

.interface-response-type-switch .body-format-switch__btn {
  padding: 3px 10px;
  height: auto;
  font-size: 12px;
  line-height: 1.2;
}

.body-none-hint {
  margin: 0;
  font-size: 13px;
  line-height: 1.85;
}

.headers-pane {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  min-width: 0;
}

.hidden-headers-toggle {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-left: 2px;
  border: 0;
  background: color-mix(in srgb, var(--color-text-secondary) 12%, transparent);
  color: var(--color-text-secondary);
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  line-height: 1.2;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.hidden-headers-toggle:hover {
  color: var(--color-interface-field-text);
  background: color-mix(in srgb, var(--color-interface-accent) 14%, transparent);
}

.hidden-headers-toggle--open {
  color: var(--color-interface-accent);
  background: color-mix(in srgb, var(--color-interface-accent) 16%, transparent);
}

.hidden-headers-toggle svg {
  flex-shrink: 0;
}

.param-table__row--auto {
  background: transparent;
}

.param-table__auto-key,
.param-table__auto-value,
.param-table__auto-desc,
.param-table__auto-spacer {
  display: flex;
  align-items: center;
  min-width: 0;
  min-height: 34px;
  margin: 3px 6px;
  padding: 0 8px;
  font-size: 13px;
  color: var(--color-text-secondary);
  border-radius: 4px;
  box-sizing: border-box;
}

.param-table__auto-key {
  gap: 6px;
  color: var(--color-interface-field-text);
}

.param-table__auto-key > span:first-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.param-table__auto-info {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  margin-left: auto;
  border: 1px solid currentColor;
  border-radius: 50%;
  font-size: 10px;
  font-style: italic;
  font-family: Georgia, 'Times New Roman', serif;
  line-height: 1;
  opacity: 0.5;
  cursor: help;
}

.param-table__auto-value {
  color: var(--color-interface-field-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.param-table__auto-spacer {
  margin: 4px 0;
  padding: 0;
}

.param-table {
  --param-table-head-height: 40px;
  border: 1px solid var(--color-interface-bar-border);
  border-radius: var(--raw-editor-selection-radius);
  overflow: hidden;
  min-height: 0;
  max-width: 100%;
  min-width: 0;
  background: var(--color-workspace-content);
}

.param-table__row {
  display: grid;
  grid-template-columns: 40px 1fr 1fr 1.2fr 40px;
  align-items: center;
  min-height: 42px;
  border-bottom: 1px solid var(--color-interface-bar-border);
  background: var(--color-workspace-content);
  transition: background-color 0.12s ease;
}

.param-table--resizable-wrap {
  position: relative;
  max-width: 100%;
  min-width: 0;
}

.param-table--resizable-wrap .param-table__row {
  position: relative;
  z-index: 1;
}

.param-table__row:last-child {
  border-bottom: 0;
}

.param-table__row:not(.param-table__row--head):hover {
  background: color-mix(in srgb, var(--color-text-secondary) 28%, var(--color-workspace-content));
}

.param-table__row--head {
  --param-table-head-height: 40px;
  height: var(--param-table-head-height);
  min-height: var(--param-table-head-height);
  background: var(--color-workspace-content);
  pointer-events: none;
  user-select: none;
}

.param-table__row--head .param-table__check {
  pointer-events: auto;
  margin: 0;
  height: 100%;
  align-self: stretch;
}

.param-table__head-cell,
.param-table__head-action {
  display: flex;
  align-items: center;
  min-width: 0;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.01em;
  color: var(--color-text-secondary);
  background: transparent;
  position: relative;
  overflow: hidden;
}

.param-table__head-action {
  padding: 0;
}

.param-table__row--off {
  opacity: 0.55;
}

.param-table__check {
  display: flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  height: 34px;
  margin: 3px 0;
  background: transparent;
  cursor: pointer;
}

.param-table__check input {
  appearance: none;
  -webkit-appearance: none;
  width: 15px;
  height: 15px;
  margin: 0;
  cursor: pointer;
  border: 1px solid color-mix(in srgb, var(--color-text-secondary) 65%, transparent);
  border-radius: var(--raw-editor-selection-radius);
  background-color: transparent;
  background-repeat: no-repeat;
  background-position: center;
  background-size: 11px 11px;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__check input:checked {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none'%3E%3Cpath d='M3.2 8.2L6.4 11.4L12.8 4.4' stroke='%23fff' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
}

.param-table__check input:indeterminate {
  background-color: var(--color-primary);
  border-color: var(--color-primary);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none'%3E%3Cpath d='M3.5 8H12.5' stroke='%23fff' stroke-width='2.4' stroke-linecap='round'/%3E%3C/svg%3E");
}

.param-table__check input:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
  outline-offset: 1px;
}

.param-table__combo {
  min-width: 0;
  height: 34px;
  margin: 3px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  overflow: hidden;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__combo {
  background: var(--color-workspace-inset);
}

.param-table__combo:focus-within {
  background: var(--color-workspace-inset);
  border-color: var(--color-primary);
}

.param-table__combo :deep(.combo-input),
.param-table__combo :deep(.combo-input__field) {
  height: 100%;
  min-height: 0;
  border-radius: 4px;
}

.param-table__combo :deep(.combo-input__field) {
  padding: 0 28px 0 10px;
  background: transparent;
}

.param-table__combo :deep(.combo-input__field:focus) {
  background: transparent;
}

.param-table__cell {
  min-width: 0;
  width: calc(100% - 12px);
  height: 34px;
  min-height: 34px;
  margin: 3px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--color-interface-field-text);
  padding: 0 10px;
  font: inherit;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__cell {
  background: var(--color-workspace-inset);
}

.param-table__cell:focus {
  background: var(--color-workspace-inset);
  border-color: var(--color-primary);
}

.param-table__value {
  position: relative;
  min-width: 0;
  height: 100%;
  min-height: 42px;
  display: flex;
  /* Keep shell at margin-top (3px), matching absolute top when focused — avoid 1px jump from centering. */
  align-items: flex-start;
}

.param-table__row:has(.param-table__value-shell:focus-within) {
  z-index: 3;
}

.param-table__cell--inner {
  flex: 1;
  min-width: 0;
  width: calc(100% - 12px);
}

.param-table__value-shell {
  flex: 1;
  min-width: 0;
  width: calc(100% - 12px);
  height: 34px;
  margin: 3px 6px;
  display: flex;
  align-items: stretch;
  box-sizing: border-box;
  border: 1px solid transparent;
  border-radius: 4px;
}

.param-table__value-shell:focus-within {
  position: absolute;
  left: 6px;
  top: 3px;
  z-index: 3;
  width: calc(100% - 12px);
  height: auto;
  min-height: 34px;
  max-height: 240px;
  margin: 0;
  border-color: var(--color-primary);
  background: var(--color-workspace-content);
  box-shadow: 0 4px 16px color-mix(in srgb, #000 28%, transparent);
  overflow: hidden;
  resize: both;
}

.param-table__value-input {
  resize: none;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  border: none;
  width: 100%;
  height: 100%;
  margin: 0;
  /* 6+20+6 = 32 matches shell content box (34 - 2px border). */
  padding: 6px 10px;
  line-height: 20px;
  min-height: 0;
  box-sizing: border-box;
}

.param-table__value-shell:focus-within .param-table__value-input {
  border: none;
  background: transparent;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  text-overflow: clip;
  /* Keep the same padding/line-height as collapsed to prevent text jump. */
  padding: 6px 10px;
  line-height: 20px;
  overflow-x: hidden;
  overflow-y: hidden;
}

.param-table__type {
  display: flex;
  align-items: center;
  min-width: 0;
  height: 100%;
  min-height: 42px;
  padding: 0;
  box-sizing: border-box;
}

.param-table__type-select {
  flex: 1;
  min-width: 0;
  display: flex;
  width: calc(100% - 12px);
  height: 34px;
  margin: 3px 6px;
  border: 1px solid transparent;
  outline: none;
  border-radius: 4px;
  overflow: hidden;
  background: transparent;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__type-select {
  background: var(--color-workspace-inset);
}

.param-table__type-select:focus-within {
  background: var(--color-workspace-inset);
  border-color: var(--color-primary);
}

.param-table__type-select :deep(.menu-select.compact.plain) {
  flex: 1;
  width: 100%;
  height: 100%;
  min-height: 0;
  border-radius: 4px;
  overflow: hidden;
}

.param-table__type-select :deep(.menu-select.compact.plain .menu-select__trigger) {
  display: flex;
  width: 100%;
  height: 100%;
  min-height: 0;
  max-height: none;
  border: none !important;
  outline: none;
  border-radius: 4px;
  background: transparent !important;
  box-shadow: none !important;
  padding: 0 4px 0 8px;
  gap: 2px;
  color: var(--color-interface-field-text);
}

.param-table__type-select :deep(.menu-select.compact.plain .menu-select__trigger:hover),
.param-table__type-select :deep(.menu-select.compact.plain.open .menu-select__trigger) {
  border: none !important;
  background: transparent !important;
  box-shadow: none !important;
}

.param-table__type-select :deep(.menu-select__label) {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 500;
}

.param-table__type-select :deep(.menu-select__chevron) {
  flex-shrink: 0;
  margin-left: 0;
}

.param-table__type-select :deep(.menu-select__chevron svg) {
  width: 12px;
  height: 12px;
}

.param-table__file {
  flex: 1;
  display: flex;
  align-items: center;
  min-width: 0;
  width: calc(100% - 12px);
  height: 34px;
  margin: 3px 6px;
  padding: 0 10px;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  overflow: hidden;
  box-sizing: border-box;
  transition:
    background-color 0.12s ease,
    border-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__file {
  background: var(--color-workspace-inset);
}

.param-table__file:focus-within {
  background: var(--color-workspace-inset);
  border-color: var(--color-primary);
}

.param-table__file-input {
  display: none;
}

.param-table__file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  color: var(--color-interface-placeholder);
}

.param-table__file-name.filled {
  color: var(--color-interface-field-text);
}

.param-table__cell::placeholder {
  color: var(--color-interface-placeholder);
  opacity: 1;
}

.param-table__actions {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  gap: 0;
  min-width: 0;
}

.param-table__actions .param-table__del {
  margin: 0;
}

.param-table__row:not(.param-table__row--head):hover :deep(.param-table__dict) {
  opacity: 1;
  pointer-events: auto;
}

.param-table__row:not(.param-table__row--head):hover :deep(.param-table__dict--matched) {
  color: var(--color-interface-accent, var(--color-primary));
  background: transparent;
}

.param-table__row:not(.param-table__row--head):hover :deep(.param-table__dict--matched:hover),
.param-table__row:not(.param-table__row--head):hover :deep(.param-table__dict--open) {
  background: color-mix(in srgb, var(--color-interface-accent, var(--color-primary)) 16%, transparent);
}

.param-table__del {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: center;
  width: 28px;
  height: 28px;
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  border-radius: var(--raw-editor-selection-radius);
  opacity: 0;
  pointer-events: none;
  position: relative;
  z-index: 2;
  /* Optical center: × glyph sits slightly low in most fonts */
  padding-bottom: 1px;
  transition:
    opacity 0.12s ease,
    color 0.12s ease,
    background-color 0.12s ease;
}

.param-table__row:not(.param-table__row--head):hover .param-table__del {
  opacity: 1;
  pointer-events: auto;
}

.param-table__del:hover:not(:disabled) {
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 16%, transparent);
}

.param-table__del:disabled {
  opacity: 0;
  cursor: not-allowed;
}

.param-table__row:not(.param-table__row--head):hover .param-table__del:disabled {
  opacity: 0.35;
  pointer-events: none;
}

.auth-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.auth-pane__layout {
  flex: 1;
  display: flex;
  align-items: stretch;
  min-height: 0;
}

.auth-pane__left {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 20px;
}

.auth-pane__type {
  width: 200px;
  max-width: 100%;
  flex-shrink: 0;
}

.auth-pane__type :deep(.menu-select) {
  width: 100%;
}

.auth-pane__divider {
  width: 1px;
  flex-shrink: 0;
  align-self: stretch;
  margin: 10px 0;
  background: var(--color-interface-bar-border);
}

.auth-pane__right {
  flex: 1;
  min-width: 0;
  max-width: 520px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 20px;
}

.auth-pane__label {
  font-size: 13px;
  font-weight: 600;
  line-height: 20px;
  color: var(--color-interface-field-text);
}

.auth-pane__label--spacer {
  visibility: hidden;
  user-select: none;
}

.auth-pane__hint--none {
  margin: 0;
  line-height: 34px;
}

.auth-pane__input {
  width: 100%;
  max-width: 520px;
}

.auth-pane__input::placeholder {
  color: var(--color-interface-placeholder);
  opacity: 1;
}

.auth-pane__hint {
  margin: 2px 0 0;
  font-size: 12px;
  line-height: 1.55;
}
</style>
