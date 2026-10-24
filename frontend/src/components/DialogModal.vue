<script setup lang="ts">
import { onMounted, ref } from 'vue'

defineProps<{ judul: string }>()
const emit = defineEmits<{ tutup: [] }>()
const dialog = ref<HTMLDialogElement>()

onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog ref="dialog" class="dialog" @cancel.prevent="emit('tutup')" @click.self="emit('tutup')">
    <div class="isi">
      <header>
        <h2>{{ judul }}</h2>
        <button type="button" class="tombol polos" aria-label="Tutup" @click="emit('tutup')">Tutup</button>
      </header>
      <slot />
    </div>
  </dialog>
</template>

<style scoped>
.dialog {
  width: min(520px, calc(100vw - 32px));
  max-height: calc(100dvh - 32px);
  padding: 0;
  border: 0;
  border-radius: var(--r);
  box-shadow: 0 20px 60px rgb(23 32 54 / 0.3);
  color: var(--tinta);
}
.dialog::backdrop {
  background: rgb(28 42 82 / 0.45);
}
.isi {
  display: grid;
  gap: 16px;
  padding: 20px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
</style>
