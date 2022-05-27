<template>
  <div class="pu-box">
    <div>Address {{ data.poolKey }}</div>
    <div>Mint A {{ data.mint_a }}</div>
    <div>Mint B {{ data.mint_b }}</div>
  </div>
</template>

<script>
export default {
  name: "PoolUnit",
  props: {
    data: {
      type: Object,
      default() {
        return {
          fee: 0.01,
          fee_vault: "4TbFgUz1faPpHQ6QyXA4Gm6vg4KjWfqDZBhfdPFgtaU6",
          ka: 100000000000,
          kb: 100000000000,
          mint_a: "GEEJqrshj3r4CbSN7fJk6haCPBTLWczaw3UGepB8hVE2",
          mint_b: "9shyAizyTSUYnQPu2hDuphv9eW17V9xJProXAghEAbv4",
          nonce: 255,
          owner: "48a9Dv7YcHCWGAkMVf6WaBeuyxhqAakwjEN5Dvb1zAGD",
          poolKey: "6F6cYDiHShEgHjmnJKzFaeuD7xq9zqtvKf5ifrmx1x8b",
          status: 1,
          tolerance: 1000,
          vault_a: "8YyM1aaMejVj7mnjqMyyeV8LRwWxETPXyGUUeTPDij49",
          vault_b: "HiRMKjQKcYawspHh8fM25trDWdRt73Mm7PBsV4xdkBuE",
        };
      },
    },
  },
  computed: {},
};
</script>

<style lang="less" scoped>
.pu-box {
  padding: 15px;
  border-bottom: solid 1px gray;
}
</style>
