<template>
  <p></p>
  <div class="group">
    <img src="../assets/xiaohui.png" class="xiaohui">
    <h1>QUIC-Based 音视频通信</h1>
  </div>
  <p></p>
  <p></p>
  <p></p>
  <form @submit.prevent="click">
    <div class="row">
      <input v-model="name" placeholder="输入用户名" style="margin-bottom: 5px;width: 65%;" />
    </div>
    <div class="row">
      <input v-model="addr" placeholder="输入服务器地址" style="width: 49%;" />
      <p style="width: 1%;"></p>
      <button type="submit" style="width: 15%;">确定</button>
    </div>
  </form>

</template>


<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { ElLoading } from 'element-plus'
import { ElMessage } from 'element-plus'
import router from "../router";

// 用户名
const name = ref("");
// 服务器地址
const addr = ref("127.0.0.1");


async function click() {
  // 显示 Loading
  const loading = ElLoading.service({
    lock: true,
    text: '测试连接',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  invoke(
    "init", { name: name.value, addr: addr.value }
  ).then((_m) => {
    ElMessage({
      message: '连接测试成功',
      type: 'success',
    });
    // 转到Do页面
    router.push("/Start");
  }).catch((e) => {
    ElMessage.error('连接错误: ' + e)
  }).finally(() => {
    // 关闭Loading
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
  /* 使用弹性布局 */
  display: flex;
  /* 垂直居中对齐 */
  align-items: center;
  /* 将子元素在水平方向上居中对齐 */
  justify-content: inherit;
  /* 将子元素在垂直方向上居中对齐 */
  align-items: inherit;
}

.xiaohui {
  /* 使图片垂直中心与文本基线对齐 */
  vertical-align: middle;
  /* 可选：为了在图片和标题之间增加一些间距 */
  margin-right: 10px;
  height: 66.88px;
}
</style>