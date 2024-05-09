<template>
    <div>
        <img style="width: 100%;" :src="img" :disable="imgdisable" />
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { ElLoading, ElMessage } from 'element-plus';
import router from '../router';

const img = ref("")
const imgdisable = ref(true)

async function play() {
    const loading = ElLoading.service({
        lock: true,
        text: '等待中',
        background: 'rgba(0, 0, 0, 0.7)',
    })



    let unlisten = await listen('wake', () => {
        loading.close();
        // imgdisable.value = false;
    });

    let unlisten2 = await listen('play_frame', (event) => {
        img.value = "data:image/jpeg;base64," + event.payload as string;
        if (imgdisable.value === true) {
            imgdisable.value = false;
        }
    });

    invoke("wait").catch(() => {
        ElMessage.error('请求错误');
        router.back();
    });
}

setTimeout(async () => {
    await play()
}, 333);

</script>