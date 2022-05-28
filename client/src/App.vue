<template>
  <div id="app">
    <el-container>
      <el-header>
        <app-header />
      </el-header>
      <el-main>
        <router-view />
      </el-main>
      <el-footer>
        <app-footer />
      </el-footer>
    </el-container>
  </div>
</template>

<script>
import AppHeader from "./layout/Header.vue";
import AppFooter from "./layout/Footer.vue";

export default {
  name: "App",
  components: {
    AppHeader,
    AppFooter,
  },
};
</script>

<style lang="less">
* {
  margin: 0;
  padding: 0;
  font-family: "Microsoft YaHei";
}
html {
  background-color: #f9f9f9;
}
body {
  max-width: 1360px;
  margin: 0 auto;
}
#app {
  .el-main {
    width: 100%;
    min-height: calc(100vh - 120px);
  }
  .ml15 {
    margin-left: 15px;
  }
  .mt25 {
    margin-top: 25px;
  }
  .none {
    font-size: 24px;
    font-weight: 500;
    text-align: center;
    line-height: 60px;
  }
}
</style>
