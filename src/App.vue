<script setup lang="ts">
// import Init from './components/Init.vue';
</script>

<template>
  <div class="container">
    <router-view></router-view>
    <!-- <Init/> -->
  </div>
</template>

<style scoped>
.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}



.xiaohui {
  vertical-align: middle;
  /* 使图片垂直中心与文本基线对齐 */
  margin-right: 10px;
  /* 可选：为了在图片和标题之间增加一些间距 */
  height: 66.88px;
}
</style>
