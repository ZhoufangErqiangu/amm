<template>
  <div class="pu-box">
    <el-form label-width="150px">
      <el-form-item label="Address">
        {{ data.poolKey }}
        <el-button
          type="danger"
          size="mini"
          class="ml15"
          v-show="isOwner"
          @click="onTerminate"
          :loading="loading4"
        >
          Terminate
        </el-button>
      </el-form-item>
      <el-form-item label="Status">
        <span v-if="data.status === 0">Not Init</span>
        <span v-else-if="data.status === 1">Nomal</span>
        <span v-else-if="data.status === 2">Lock</span>
        <span v-else>Unknown</span>
        <el-button
          type="warning"
          size="mini"
          class="ml15"
          v-show="isOwner && data.status != 1"
          @click="onChangeStatus(1)"
          :loading="loading2"
        >
          Nomal
        </el-button>
        <el-button
          type="warning"
          size="mini"
          class="ml15"
          v-show="isOwner && data.status != 2"
          @click="onChangeStatus(2)"
          :loading="loading2"
        >
          Lock
        </el-button>
      </el-form-item>
      <el-form-item label="Mint A">{{ data.mint_a }}</el-form-item>
      <el-form-item label="Mint B">{{ data.mint_b }}</el-form-item>
      <el-form-item label="Fee Rate">{{ data.fee * 100 }} %</el-form-item>
      <el-form-item label="Fee Amount" v-show="isOwner">
        {{ feeAmount }}
        <el-button
          type="primary"
          size="mini"
          class="ml15"
          @click="onWithdrawalFee"
          :loading="loading3"
        >
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
    <el-form label-width="150px">
      <el-form-item label="Simulate">
        You will
        <span
          v-for="(item, index) in options"
          :key="index"
          v-show="option.direction == item.value"
        >
          {{ item.description }}
        </span>
        <span>{{ simulateAmount.toFixed(3) }}</span>
      </el-form-item>
      <el-form-item label="Simulate Rate">
        <span>{{ simulateRate.toFixed(3) }}</span>
      </el-form-item>
    </el-form>
  </div>
</template>

<script>
import store from "../../store";
import { Connection } from "@solana/web3.js";
import { rpcUrl } from "../../assets/js";
import {
  swap,
  terminate,
  updateStatus,
  withdrawalFee,
} from "../../assets/js/amm";
import { wallet } from "../../plugin/wallet";
import { getTokenAccountData } from "../../assets/js/amm/lib/tokenAccount";
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
      vaultAData: 0,
      vaultBData: 0,
      feeAmount: 0,
      option: {
        amount: 0,
        direction: 1,
      },
      options: [
        { label: "A2B", value: 1, description: "receive" },
        { label: "B2A", value: 2, description: "pay" },
      ],
      loading: false,
      loading2: false,
      loading3: false,
      loading4: false,
    };
  },
  methods: {
    async getData() {
      {
        let res = await getTokenAccountData(connection, this.data.vault_a);
        if (res.code == 1) {
          this.vaultAData = res.data;
        } else {
          console.error("get vault a amount", res);
        }
      }
      {
        let res = await getTokenAccountData(connection, this.data.vault_b);
        if (res.code == 1) {
          this.vaultBData = res.data;
        } else {
          console.error("get vault b amount", res);
        }
      }
      {
        let res = await getTokenAccountData(connection, this.data.fee_vault);
        if (res.code == 1) {
          this.feeAmount = res.data.amount;
        } else {
          console.error("get fee vault amount", res);
        }
      }
    },
    async onSwap() {
      if (!(parseFloat(this.option.amount) > 0)) {
        this.$message({ message: "Must input valid amount.", type: "warning" });
        return;
      }
      this.loading = true;
      try {
        let res = await swap(
          connection,
          wallet,
          this.data.poolKey,
          parseFloat(this.option.amount),
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
    async onChangeStatus(status) {
      if (!this.isOwner) {
        this.$message({ message: "Must be owner.", type: "warning" });
        return;
      }
      this.loading2 = true;
      try {
        let res = await updateStatus(
          connection,
          wallet,
          this.data.poolKey,
          status
        );
        if (res.code == 1) {
          this.$message({ message: "Change status OK", type: "success" });
        } else {
          console.warn("change status fail", res);
          this.$message({ message: "Change status fail", type: "warning" });
        }
      } catch (err) {
        console.error("change status error", err);
        this.$message({ message: "Change status error", type: "error" });
      }
      this.loading2 = false;
    },
    async onWithdrawalFee() {
      if (!this.isOwner) {
        this.$message({ message: "Must be owner.", type: "warning" });
        return;
      }
      this.loading3 = true;
      try {
        let res = await withdrawalFee(connection, wallet, this.data.poolKey);
        if (res.code == 1) {
          this.$message({ message: "Withdrawal OK", type: "success" });
        } else {
          console.warn("withdrawal fail", res);
          this.$message({ message: "Withdrawal fail", type: "warning" });
        }
      } catch (err) {
        console.error("withdrawal error", err);
        this.$message({ message: "Change error", type: "error" });
      }
      this.loading3 = false;
    },
    async onTerminate() {
      if (!this.isOwner) {
        this.$message({ message: "Must be owner.", type: "warning" });
        return;
      }
      this.loading4 = true;
      try {
        let res = await terminate(connection, wallet, this.data.poolKey);
        if (res.code == 1) {
          this.$message({ message: "Withdrawal OK", type: "success" });
        } else {
          console.warn("withdrawal fail", res);
          this.$message({ message: "Withdrawal fail", type: "warning" });
        }
      } catch (err) {
        console.error("withdrawal error", err);
        this.$message({ message: "Change error", type: "error" });
      }
      this.loading4 = false;
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
    simulateAmount() {
      let k =
        (this.data.ka * this.data.kb) /
        10 ** this.vaultAData.decimals /
        10 ** this.vaultBData.decimals;
      if (this.option.direction === 1) {
        // a2b
        // (a+da)*(b-db)=k
        return (
          this.vaultBData.amount -
          k / (this.vaultAData.amount + parseFloat(this.option.amount))
        );
      } else if (this.option.direction == 2) {
        // b2a
        // (a-da)*(b+db)=k
        return (
          k / (this.vaultAData.amount - parseFloat(this.option.amount)) -
          this.vaultBData.amount
        );
      } else {
        return 0;
      }
    },
    simulateRate() {
      if (parseFloat(this.option.amount) > 0) {
        return this.simulateAmount / parseFloat(this.option.amount);
      } else {
        return 0;
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
