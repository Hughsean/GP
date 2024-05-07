<template>
    <div class="row">
        <img style="width: 100%;" :src="img" :disable="imgdisable" />
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import router from '../router';
import { ElMessage } from 'element-plus';

const play_disable = ref(false)
const img = ref("")
const imgdisable = ref(true)


async function play() {
    play_disable.value = true;

    invoke("call").catch(() => {
        ElMessage.error('请求错误');
        router.back();
    });

    let unlisten = await listen('play_frame', (event) => {
        img.value = "data:image/jpeg;base64," + event.payload as string;
        if (imgdisable.value === true) {
            imgdisable.value = false;
        }
    });
}

setTimeout(async () => {
    await play();
    imgdisable.value = false;
}, 333);

</script>