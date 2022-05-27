<template>
  <div class="head-box">
    <h1>AMM UI</h1>
    <el-button
      class="btn"
      type="primary"
      @click="onConnect"
      :loading="loading"
      :disabled="isConnected"
      v-show="!isConnected"
    >
      Connect
    </el-button>
    <el-button
      class="btn"
      type="primary"
      @click="onDisconnect"
      :loading="loading"
      :disabled="!isConnected"
      v-show="isConnected"
    >
      Disconnect
    </el-button>
  </div>
</template>

<script>
import { connectWallet, disconnectWallet } from '../plugin/wallet';

export default {
  name: 'Head',
  data() {
    return {
      loading: false,
      isConnected: false,
    };
  },
  methods: {
    async onConnect() {
      this.loading = true;
      try {
        this.isConnected = await connectWallet();
        this.loading = false;
      } catch (err) {
        this.loading = false;
        console.error('wallet connect error', err);
      }
    },
    async onDisconnect() {
      this.loading = true;
      try {
        this.isConnected = await disconnectWallet();
        this.loading = false;
      } catch (err) {
        this.loading = false;
        console.error('wallet connect error', err);
      }
    },
  },
};
</script>

<style lang="less" scoped>
.head-box {
  position: relative;
  h1 {
    font-size: 36px;
    font-weight: 700;
    line-height: 60px;
    text-align: center;
  }
  .btn {
    position: absolute;
    top: 50%;
    right: 15px;
    transform: translateY(-50%);
  }
}
</style>
