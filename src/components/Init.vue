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
  <!-- <form> -->

  <!-- <div style="height: 1%;">-</div> -->
  <div class="row">
    <input v-model="addr" placeholder="输入服务器地址" style="width: 49%;margin-right: 1%;margin-bottom: 1%;" />
    <!-- <p style="width: 1%;"></p> -->
    <button @click="init" style="width: 15%;margin-bottom: 1%;">确定</button>
  </div>
  <!-- </form> -->
  <div class="row">
    <el-radio-group size="large" v-model="mode" @change="change" style=" width: 20%;">
      <el-radio-button label="呼叫" value="Call" />
      <el-radio-button label="等待" value="Wait" />
    </el-radio-group>

    <input v-model="wait_name" placeholder="请输入您的昵称" v-show="!select_disable" style="width: 45%;" />

    <el-select v-model="call_name" placeholder="选择被呼叫用户" style="width: 29%;margin-right: 1%;" v-show="select_disable"
      size="large">
      <el-option v-for="item in options" :key="item" :label="item" :value="item" />
    </el-select>
    <button style="width: 15%;" v-show="select_disable" @click="refresh">刷新</button>

  </div>
</template>


<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { ElLoading } from 'element-plus'
import { ElMessage } from 'element-plus'
import router from "../router";
import { once } from "@tauri-apps/api/event";


// 用户名
const wait_name = ref("");
const call_name = ref("");
// 服务器地址
const addr = ref("127.0.0.1");

const mode = ref("Call");

const select_disable = ref(true);

const options = ref<[string]>()

const change = () => {
  if (mode.value === "Call") {
    // placeholder.value = "输入被呼叫用户名"
    select_disable.value = true;
  } else {
    // placeholder.value = "输入您的用户名"
    select_disable.value = false;
  }
}

async function refresh() {
  console.log(call_name.value);
  call_name.value = '';

  const loading = ElLoading.service({
    lock: true,
    text: '获取数据中',
    background: 'rgba(0, 0, 0, 0.7)',
  })



  let unlisten1 = await once('query', (e) => {
    unlisten2();
    options.value = e.payload as [string];
    loading.close();
  });

  let unlisten2 = await once('err', (e) => {
    unlisten1();
    ElMessage.error('错误: ' + e.payload);
    loading.close();
  });

  await invoke(
    "query", { addr: addr.value }
  );
  // .then((ret) => {
  //   ElMessage({
  //     message: '查询成功',
  //     type: 'success',
  //   });
  //   options.value = ret as [string];
  // }).catch((e) => {
  //   ElMessage.error('错误: ' + e)
  // });
}

async function init() {
  let name;

  if (mode.value === "Call") {
    name = call_name;
  } else {
    name = wait_name;
  }

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


  let unlisten1 = await once('init', () => {
    unlisten2();
    loading.close();
    ElMessage({
      message: '初始化成功',
      type: 'success',
    });
    router.replace("/" + mode.value);
  });

  let unlisten2 = await once('err', (e) => {
    unlisten1();
    ElMessage.error('错误: ' + e.payload);
    loading.close();
  });

  await invoke(
    "init", { addr: addr.value, name: name.value }
  );
  // .then(() => {
  //   ElMessage({
  //     message: '连接测试成功',
  //     type: 'success',
  //   });
  //   // 跳转页面
  //   router.push("/" + mode.value);
  // }).catch((e) => {
  //   ElMessage.error('错误: ' + e)
  // }).finally(() => {
  //   // 关闭Loading
  //   loading.close()
  // });

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