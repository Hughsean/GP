<template>
    <div>
        <img style="width: 100%;" :src="img" />
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { ElLoading } from 'element-plus';
const play_disable = ref(false)
const img = ref("https://i1.wp.com/gelatologia.com/wp-content/uploads/2020/07/placeholder.png?ssl=1")


async function play() {
    play_disable.value = true;
    const loading = ElLoading.service({
        lock: true,
        text: '等待中',
        background: 'rgba(0, 0, 0, 0.7)',
    })

    await invoke("wait");
    let unlisten = await listen('wake', () => {
        loading.close();
    });

    let unlisten2 = await listen('play_frame', (event) => {
        img.value = "data:image/jpeg;base64," + event.payload as string;
    });
}

setTimeout(async () => {
    await play()
}, 333);

</script>