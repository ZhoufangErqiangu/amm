<template>
  <div class="p-unit">
    <div class="date">
      {{ dateStr }}
    </div>
    <div class="user">
      {{ authorStr }} Says:
    </div>
    <div class="content">
      {{ data.content }}
    </div>
  </div>
</template>

<script>
export default {
  name: "PostUnit",
  props: {
    data: {
      type: Object,
      default() {
        return {
          author: "",
          content: "",
          key: "",
          master: "",
          postType: 1,
          postTypeLabel: "Master",
          quote: "",
          sharding: 0,
          time: 0,
        };
      },
    },
  },
  computed: {
    authorStr() {
      let author = this.data.author;
      if (author.length > 10) {
        return `${author.slice(0, 5)}...${author.slice(-5)}`;
      } else {
        return author;
      }
    },
    dateStr() {
      return this.$moment(this.data.time).fromNow();
    },
  },
};
</script>

<style lang="less" scoped>
.p-unit {
  margin-bottom: 15px;
  .content{
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .date {
    float: right;
    color: #777;
  }
}
</style>