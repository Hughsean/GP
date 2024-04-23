<template>
  <p></p>
  <div class="group">
    <img src="../assets/xiaohui.png" class="xiaohui">
    <h1>QUIC-Based 音视频通信</h1>
  </div>
  <p></p>
  <p></p>
  <p></p>
  <form @submit.prevent="fun">
    <div class="row">
      <input v-model="name" placeholder="输入用户名" style="margin-bottom: 5px;width: 65%;" />
    </div>
    <div class="row">
      <input v-model="addr" placeholder="输入服务器地址" style="width: 49%;" />
      <p style="width: 1%;"></p>
      <button type="submit" style="width: 15%;">确定</button>
      <!-- <el-button type="submit" @click="fun"> 确定 </el-button> -->
    </div>
  </form>
  <p>{{ greetMsg }}</p>

</template>
<script setup lang="ts">

import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { ElLoading } from 'element-plus'
import { ElMessage } from 'element-plus'

const greetMsg = ref("");
const name = ref("");
const addr = ref("");

const _Sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}


async function fun() {
  // greetMsg.value = await invoke("fun", { name: name.value, n: 0 });
  const loading = ElLoading.service({
    lock: true,
    text: 'Loading',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  invoke(
    "init", { name: name.value, addr: addr.value }
  ).then((_m) => {
    ElMessage({
      message: '初始化成功',
      type: 'success',
    })
  }).catch((e) => {
    ElMessage.error('错误: ' + e)
  }).finally(() => {
    loading.close()
  });
}



</script>


<style scoped>
.row {
  display: flex;
  justify-content: center;
  text-align: center;
  margin: 0;
}

.group {
  display: flex;
  /* 使用弹性布局 */
  align-items: center;
  /* 垂直居中对齐 */
  justify-content: inherit;
  /* 将子元素在水平方向上居中对齐 */
  align-items: inherit;
  /* 将子元素在垂直方向上居中对齐 */
}

.xiaohui {
  vertical-align: middle;
  /* 使图片垂直中心与文本基线对齐 */
  margin-right: 10px;
  /* 可选：为了在图片和标题之间增加一些间距 */
  height: 66.88px;
}
</style>