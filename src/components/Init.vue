<template>
  <p></p>
  <p></p>
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
      <el-radio-group v-model="mode" size="large" @change="change" style="margin-right: 5%;margin-bottom: 1%;">
        <el-radio-button label="呼叫" value="Call" />
        <el-radio-button label="等待" value="Wait" />
      </el-radio-group>

      <input v-model="name" :placeholder="placeholder" style="margin-bottom: 5px;width: 41%;" />

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

const mode = ref("Call")
const placeholder = ref("输入被呼叫用户名")

const change = () => {
  // name.value = ""
  if (mode.value === "Call") {
    placeholder.value = "输入被呼叫用户名"
  } else {
    placeholder.value = "输入您的用户名"
  }
}
async function click() {
  if (name.value.length === 0) {
    ElMessage.error('请输入用户名');
    return
  }
  if (addr.value.length === 0) {
    ElMessage.error('请输入地址');
    return
  }

  // 显示 Loading
  const loading = ElLoading.service({
    lock: true,
    text: '测试连接并初始化设备',
    background: 'rgba(0, 0, 0, 0.7)',
  })

  invoke(
    "init", { addr: addr.value, name: name.value }
  ).then(() => {
    ElMessage({
      message: '连接测试成功',
      type: 'success',
    });
    // 跳转页面

    router.push("/" + mode.value);


  }).catch((e) => {
    ElMessage.error('错误' + e)
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