<template>
    <div class="row">
        <img style="height: 440px;" :src="img" fit="contain" />
    </div>
    <!-- <div class="row">
        <div class="row">
            <el-button type="success" :disabled="play_disable" @click="play">开始</el-button>
        </div>
    </div> -->
</template>

<script setup lang="ts">
import { ComponentOptionsBase, ComponentPublicInstance, Ref, ref } from 'vue';
import { UnlistenFn, emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { ElLoading, LoadingParentElement } from 'element-plus';
const play_disable = ref(false)
const img = ref("https://i1.wp.com/gelatologia.com/wp-content/uploads/2020/07/placeholder.png?ssl=1")


async function play() {
    play_disable.value = true;

    await invoke("call");


    let unlisten = await listen('play_frame', (event) => {

        img.value = "data:image/jpeg;base64," + event.payload as string;
    });
}

setTimeout(async () => {
    await play()
}, 333);

</script>