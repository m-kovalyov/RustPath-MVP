<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { EditorView, keymap } from '@codemirror/view'
import { EditorState, Compartment } from '@codemirror/state'
import { basicSetup } from 'codemirror'
import { rust } from '@codemirror/lang-rust'
import { oneDark } from '@codemirror/theme-one-dark'
import { indentWithTab } from '@codemirror/commands'
const props = defineProps<{modelValue:string; disabled?:boolean}>()
const emit = defineEmits<{ 'update:modelValue':[string]; run:[] }>()
const host = ref<HTMLElement>()
let view: EditorView | undefined
const editable = new Compartment()
onMounted(() => {
  view = new EditorView({ parent:host.value, state: EditorState.create({doc:props.modelValue,extensions:[
    basicSetup, rust(), oneDark, EditorView.lineWrapping,
    editable.of([EditorView.editable.of(!props.disabled), EditorState.readOnly.of(!!props.disabled)]),
    EditorView.contentAttributes.of({'aria-label':'Редактор Rust-кода','data-testid':'code-input','aria-describedby':'editor-help'}),
    keymap.of([{key:'Mod-Enter',run:()=>{emit('run'); return true}},indentWithTab]),
    EditorView.updateListener.of(update=>{if(update.docChanged) emit('update:modelValue',update.state.doc.toString())})
  ]}) })
})
watch(()=>props.modelValue, value=>{ if(view && value!==view.state.doc.toString()) view.dispatch({changes:{from:0,to:view.state.doc.length,insert:value}}) })
watch(()=>props.disabled, disabled=>view?.dispatch({effects:editable.reconfigure([EditorView.editable.of(!disabled),EditorState.readOnly.of(!!disabled)])}))
onBeforeUnmount(()=>view?.destroy())
</script>
<template><div ref="host" class="editor-host"></div></template>
