<template>
    <h1>Todo</h1>
    <div class="row">
        <button @click="back">Play Test</button>
    </div>
</template>

<script setup lang="ts">
import router from '../router';

async function back() {
    router.push("/Play");
}
</script>