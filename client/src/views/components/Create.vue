<template>
  <div class="c-box">
    <el-card class="mt25">
      <div slot="header">
        Create Pool
        <el-button
          class="ml15"
          size="mini"
          type="primary"
          @click="onCreate"
          :loading="loading"
          >Create</el-button
        >
      </div>
      <el-form
        :rules="rules"
        :model="option"
        label-width="150px"
        @validate="onValidate"
      >
        <el-form-item prop="mint" label="Mint A Address">
          <el-input
            v-model="option.mintA"
            placeholder="Mint Address"
            clearable
          ></el-input>
        </el-form-item>
        <el-form-item prop="amount" label="Mint A Amount">
          <el-input
            v-model="option.amountA"
            placeholder="Amount"
            type="number"
          ></el-input>
        </el-form-item>
        <el-form-item prop="mint" label="Mint B Address">
          <el-input
            v-model="option.mintB"
            placeholder="Mint Address"
            clearable
          ></el-input>
        </el-form-item>
        <el-form-item prop="amount" label="Mint B Amount">
          <el-input
            v-model="option.amountB"
            placeholder="Amount"
            type="number"
          ></el-input>
        </el-form-item>
        <el-form-item prop="mint" label="Fee Mint Address">
          <el-input
            v-model="option.feeParams.mint"
            placeholder="Mint Address"
            clearable
          ></el-input>
        </el-form-item>
        <el-form-item prop="rate" label="Fee Rate">
          <el-input
            v-model="option.feeParams.rate"
            placeholder="Fee Rate"
            type="number"
          ></el-input>
        </el-form-item>
        <!-- <el-form-item prop="tolerance" label="Tolerance">
          <el-input v-model="option.tolerance" placeholder="Tolerance" type="number"></el-input>
        </el-form-item> -->
      </el-form>
    </el-card>
  </div>
</template>

<script>
import store from "../../store";
import { Connection } from "@solana/web3.js";
import { rpcUrl } from "../../assets/js";
import { createPool } from "../../assets/js/amm";
import { wallet } from "../../plugin/wallet";
const connection = new Connection(rpcUrl);

export default {
  name: "Create",
  data() {
    return {
      option: {
        feeParams: {
          mint: "",
          rate: 0.01,
        },
        amountA: 100,
        amountB: 100,
        tolerance: 1000,
        mintA: "",
        mintB: "",
      },
      rules: {
        mint: [{ require: true, message: "Must input Mint", trigger: "blur" }],
        amount: [
          { require: true, message: "Must input amount", trigger: "blur" },
          {
            min: 1,
            message: "Amount must be bigger than Zero",
            trigger: "blur",
          },
        ],
        tolerance: [
          { require: true, message: "Must input tolerance", trigger: "blur" },
        ],
        rate: [
          { require: true, message: "Must input fee rate", trigger: "blur" },
          {
            max: 1,
            message: "Fee rate must be bigger than Zero, and smaller than 1.",
            trigger: "blur",
          },
        ],
      },
      loading: false,
      validateOK: false,
    };
  },
  methods: {
    async onCreate() {
      if (!this.isConnected) {
        this.$message({ message: "Must connect wallet", type: "warning" });
        return;
      }
      if (!this.validateOK) {
        this.$message({ message: "Must input option", type: "warning" });
        return;
      }
      this.loading = true;
      try {
        let res = await createPool(
          connection,
          wallet,
          this.option.feeParams,
          this.option.amountA,
          this.option.amountB,
          this.option.tolerance,
          this.option.mintA,
          this.option.mintB
        );
        if (res.code == 1) {
          this.$message({ message: "Create ok", type: "success" });
        } else {
          console.warn("create fail", res);
          this.$message({ message: "Create fail", type: "warning" });
        }
      } catch (err) {
        console.error("create error", err);
        this.$message({ message: "Create error", type: "error" });
      }
      this.loading = false;
    },
    onValidate(value, pass, err) {
      this.validateOK = pass;
      if (err) console.warn("form validate fail", err);
    },
  },
  computed: {
    isConnected() {
      return store.state.connected;
    },
  },
};
</script>
