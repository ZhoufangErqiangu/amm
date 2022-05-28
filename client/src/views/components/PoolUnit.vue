<template>
  <div class="pu-box">
    <el-form label-width="150px">
      <el-form-item label="Address">{{ data.poolKey }}</el-form-item>
      <el-form-item label="Status">
        <span v-if="data.status === 0">Not Init</span>
        <span v-else-if="data.status === 1">Nomal</span>
        <span v-else-if="data.status === 2">Lock</span>
        <span v-else>Unknown</span>
        <el-button type="primary" size="mini" class="ml15" v-show="isOwner">
          Change
        </el-button>
      </el-form-item>
      <el-form-item label="Mint A">{{ data.mint_a }}</el-form-item>
      <el-form-item label="Mint B">{{ data.mint_b }}</el-form-item>
      <el-form-item label="Fee Rate">{{ data.fee * 100 }} %</el-form-item>
      <el-form-item label="Fee Amount" v-show="isOwner">
        {{ feeAmount }}
        <el-button type="primary" size="mini" class="ml15">
          Withdrawal Fee
        </el-button>
      </el-form-item>
    </el-form>
    <el-form :model="option" label-width="150px" :inline="true">
      <el-form-item label="Swap Amount">
        <el-input
          v-model="option.amount"
          placeholder="Amount"
          type="number"
        ></el-input>
      </el-form-item>
      <el-form-item label="Direction">
        <el-select v-model="option.direction">
          <el-option
            v-for="item in options"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          ></el-option>
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="onSwap" :loading="loading">
          Swap
        </el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<script>
import store from "../../store";
import { Connection } from "@solana/web3.js";
import { rpcUrl } from "../../assets/js";
import { swap } from "../../assets/js/amm";
import { wallet } from "../../plugin/wallet";
const connection = new Connection(rpcUrl);

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
  data() {
    return {
      feeAmount: 0,
      option: {
        amount: 0,
        direction: 1,
      },
      options: [
        { label: "A2B", value: 1 },
        { label: "B2A", value: 2 },
      ],
      loading: false,
    };
  },
  methods: {
    async getData() {},
    async onSwap() {
      if (!(this.option.amount > 0)) {
        this.$message({ message: "Must input valid amount.", type: "warning" });
        return;
      }
      this.loading = true;
      try {
        let res = await swap(
          connection,
          wallet,
          this.data.poolKey,
          this.option.amount,
          this.option.direction
        );
        if (res.code == 1) {
          this.$message({ message: "Swap OK", type: "success" });
        } else {
          console.warn("swap fail", res);
          this.$message({ message: "Swap fail", type: "warning" });
        }
      } catch (err) {
        console.error("swap error", err);
        this.$message({ message: "Swap error", type: "error" });
      }
      this.loading = false;
    },
  },
  mounted() {
    this.getData();
  },
  computed: {
    walletKey() {
      return store.state.publicKey;
    },
    isOwner() {
      return this.walletKey === this.data.owner;
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
