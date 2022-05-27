<template>
  <div class="f-box">
    <el-card>
      <div slot="header">
        <span>Find Pool</span>
        <el-button class="ml15" size="mini" type="primary" :loading="loading" @click="onFind">
          Find
        </el-button>
      </div>
      <div>
        <el-form :rules="rules" :model="option" label-width="150px" @validate="onValidate">
          <el-form-item prop="mint" label="Mint A Address">
            <el-input v-model="option.mintA" placeholder="Mint Address" clearable></el-input>
          </el-form-item>
          <el-form-item prop="mint" label="Mint B Address">
            <el-input v-model="option.mintB" placeholder="Mint Address" clearable></el-input>
          </el-form-item>
        </el-form>
      </div>
    </el-card>
    <el-card class="mt25">
      <div slot="header">
        <span>Pool List</span>
      </div>
      <div class="none" v-show="list.length === 0">No Data</div>
      <pool-unit v-for="(item, index) in list" :key="index" :data="item"></pool-unit>
    </el-card>
  </div>
</template>

<script>
import PoolUnit from "./PoolUnit.vue";
import { Connection } from "@solana/web3.js";
import { rpcUrl } from "../../assets/js";
import { findPoolByMints, getPoolsData } from "../../assets/js/amm";
const connection = new Connection(rpcUrl);

export default {
  name: "Find",
  components: {
    PoolUnit,
  },
  data() {
    return {
      loading: false,
      validateOK: false,
      list: [],
      option: {
        mintA: "",
        mintB: "",
      },
      rules: {
        mint: [{ require: true, message: "Must input Mint", trigger: "blur" }],
      },
    };
  },
  methods: {
    async onFind() {
      this.loading = true;
      try {
        if (this.mintA == "" || !this.option.mintA) {
          this.message({ message: "Must input mint address", type: "warning" });
        }
        if (this.mintB == "" || !this.option.mintB) {
          this.message({ message: "Must input mint address", type: "warning" });
        }
        let list = [];
        {
          let res = await findPoolByMints(connection, this.option.mintA, this.option.mintB);
          list = list.concat(res);
        }
        {
          let res = await findPoolByMints(connection, this.option.mintB, this.option.mintA);
          list = list.concat(res);
        }
        this.$message({ message: "Find ok", type: "success" });
        this.list = getPoolsData(list);
      } catch (err) {
        console.error("find error", err);
        this.$message({ message: "Find error", type: "error" });
      }
      this.loading = false;
    },
    onValidate(value, pass, err) {
      this.validateOK = pass;
      if (err) console.warn("form validate fail", err);
    },
  },
  computed: {},
};
</script>
