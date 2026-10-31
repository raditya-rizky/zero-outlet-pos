<script setup lang="ts">
import { onMounted, ref, useId } from 'vue'

defineProps<{ judul: string }>()
const emit = defineEmits<{ tutup: [] }>()
const dialog = ref<HTMLDialogElement>()
const idJudul = useId()

onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog
    ref="dialog"
    :aria-labelledby="idJudul"
    class="m-auto max-h-[calc(100dvh-32px)] w-[min(520px,calc(100vw-32px))] rounded-sedang border-0 p-0 text-tinta shadow-dialog backdrop:bg-indigo/45"
    @cancel.prevent="emit('tutup')"
    @click.self="emit('tutup')"
  >
    <div class="grid gap-4 p-5">
      <header class="flex items-center justify-between gap-3">
        <h2 :id="idJudul">{{ judul }}</h2>
        <button type="button" class="min-h-11 rounded-kecil px-2 font-semibold text-denim hover:bg-denim-terang" aria-label="Tutup" @click="emit('tutup')">
          Tutup
        </button>
      </header>
      <slot />
    </div>
  </dialog>
</template>
