<script setup lang="ts">

import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const greetMsg = ref("");
const name = ref("");


const Sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}


async function fun() {
  greetMsg.value = await invoke("fun", { name: name.value, n: 0 });

}



</script>

<template>
  <div>

    <form @submit.prevent="fun">
      <div class="row">
        <input v-model="name" placeholder="输入用户名" style="margin-bottom: 5px;width: 75%;" />
      </div>
      <div class="row">
        <input v-model="name" placeholder="输入服务器地址" style="width: 59%;" />
        <p style="width: 1%;"></p>
        <button type="submit" style="width: 15%;">确定</button>
      </div>
    </form>
    <p>{{ greetMsg }}</p>

  </div>

</template>


<style scoped>
.row {
  display: flex;
  /* justify-content: center; */
  justify-content: center;
  text-align: center;
  margin: 0;
}
</style>