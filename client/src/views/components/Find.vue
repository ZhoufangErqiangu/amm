<template>
  <div class="f-box">
    <el-card>
      <div slot="header">
        <span>Find Pool</span>
        <el-button class="ml15" size="mini" type="primary" :loading="loading" @click="onFind">
          Find
        </el-button>
        <el-button class="ml15" size="mini" type="primary" @click="onCreate"> Create </el-button>
      </div>
      <div>
        <el-input v-model="mintA" placeholder="Mint Address" clearable></el-input>
        <el-input v-model="mintB" placeholder="Mint Address" clearable></el-input>
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
import PoolUnit from './PoolUnit.vue';
import { Connection } from '@solana/web3.js';
import { rpcUrl } from '../../assets/js';
import { findPoolByMints } from '../../assets/js/amm';
const connection = new Connection(rpcUrl);

export default {
  name: 'Find',
  components: {
    PoolUnit,
  },
  data() {
    return {
      loading: false,
      list: [],
      mintA: '',
      mintB: '',
    };
  },
  methods: {
    async onFind() {
      this.loading = true;
      try {
        if (this.mintA == '' || !this.mintA) {
          this.message({ message: 'Must input mint address', type: 'warning' });
        }
        if (this.mintB == '' || !this.mintB) {
          this.message({ message: 'Must input mint address', type: 'warning' });
        }
        let list = [];
        {
          let res = await findPoolByMints(connection, this.mintA, this.mintB);
          list = list.concat(res);
        }
        {
          let res = await findPoolByMints(connection, this.mintB, this.mintA);
          list = list.concat(res);
        }
        this.$message({ message: 'Find ok', type: 'success' });
        this.list = list;
      } catch (err) {
        console.error('find error', err);
        this.$message({ message: 'Find error', type: 'error' });
      }
      this.loading = false;
    },
  },
  computed: {},
};
</script>
