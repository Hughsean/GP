<template>
    <h1>Play</h1>
    <!-- <div class="row">
        <img style="height: 80%;" :src="img" fit="contain" />
    </div>
    <div class="row">
        <el-button type="success" :disabled="play_disable" @click="play">Play</el-button>
        <el-button type="danger" :disabled="!play_disable" @click="close">Close</el-button>
    </div> -->


</template>

<script setup lang="ts">
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';




let stop = ref(false)
let img = ref("")
let play_disable = false

async function play() {
    play_disable = true;
    while (!stop.value) {
        invoke("play").then(() => {
        }).catch(() => {
            stop.value = true;
        });
        const unlisten = await listen('play_frame', (event) => {
            img.value = "data:image/jpeg;base64," + event.payload as string;
        });
    }
}

async function close() {
    stop.value = true;
}

</script>