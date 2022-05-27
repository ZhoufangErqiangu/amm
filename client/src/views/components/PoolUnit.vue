<template>
  <div class="pu-box">
    <div>Mint A {{ mintAShow }}</div>
    <div>Mint B {{ mintBShow }}</div>
  </div>
</template>

<script>
export default {
  name: 'PoolUnit',
  props: {
    data: {
      type: Object,
      default() {
        return {
          key: '',
          mintA: '',
          mintB: '',
        };
      },
    },
  },
  computed: {
    mintAShow() {
      if (!this.data.mintA || this.data.mintA == '') {
        return 'Null';
      } else {
        return `${this.data.mintA.slice(0, 5)}...${this.data.mintA.slice(-5)}`;
      }
    },
    mintBShow() {
      if (!this.data.mintB || this.data.mintB == '') {
        return 'Null';
      } else {
        return `${this.data.mintB.slice(0, 5)}...${this.data.mintB.slice(-5)}`;
      }
    },
  },
};
</script>

<style lang="less" scoped>
.pu-box {
  padding: 15px;
  border-bottom: solid 1px gray;
}
</style>
