<script setup lang="ts">

import Start from './components/Start.vue';
</script>

<template>
  <div class="container">
    <div class="group">
      <img src="./assets/xiaohui.png" class="xiaohui">
      <h1>QUIC-Based 音视频通信</h1>

    </div>
    <p></p>
    <Start />
  </div>
</template>

<style scoped>
.group {
    display: flex; /* 使用弹性布局 */
    align-items: center; /* 垂直居中对齐 */
    justify-content: inherit; /* 将子元素在水平方向上居中对齐 */
    align-items: inherit; /* 将子元素在垂直方向上居中对齐 */
  }

  .xiaohui {
    vertical-align: middle; /* 使图片垂直中心与文本基线对齐 */
    margin-right: 10px; /* 可选：为了在图片和标题之间增加一些间距 */
    height:66.88px;
  }
</style>
