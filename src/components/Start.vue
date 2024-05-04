<template>
    <h1>开始</h1>

    <div>
        <form @submit.prevent="submit">
            <div class="row">
                <el-radio-group v-model="mode" size="large" @change="change">
                    <el-radio-button label="呼叫" value="call" />
                    <el-radio-button label="等待" value="wait" />

                </el-radio-group>

                <input :disabled="mode === 'wait'" v-model="name" :placeholder="placeholder"
                    style="margin-bottom: 5px;width: 25%;" />
            </div>

            <p></p>
            <p></p>
            <p></p>
            <p></p>
            <div class="row">
                <button type="submit" style="width: 15%;margin-right: 15%;">确定</button>
                <!-- <p style="width: 3%;"></p> -->

                <button @click="back" style="width: 15%;margin-left: 15%;">后退</button>
            </div>
        </form>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import router from '../router';

const name = ref("")
const mode = ref("call")
const placeholder = ref("请输入用户名")

const change = () => {
    if (mode.value === "call") {
        placeholder.value = "请输入用户名"
    } else {
        placeholder.value = "等待模式不可用"
        name.value = ""
    }
}

async function submit() {

}

async function back() {
    router.back();
}
</script>
