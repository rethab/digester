<template>
  <div>
    <v-container>
      <v-list two-line dense class="py-0">
        <template v-for="(update, index) in updates">
          <ShowUpdate :key="index" :value="update" />
        </template>
      </v-list>
      <InfiniteLoading @infinite="infiniteHandler">
        <div slot="spinner">
          <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
        </div>
        <div slot="no-more"></div>
        <div slot="no-results">
          <p>There don't seem to be any updates for you :(</p>
          <p>
            Do you have subscriptions? If not, you can create them over here:
            <v-btn to="/subs" text outlined small class="mr-1" color="secondary">subscriptions</v-btn>
          </p>
        </div>
      </InfiniteLoading>
    </v-container>
  </div>
</template>

<script>
import InfiniteLoading from "vue-infinite-loading";
import ShowUpdate from "@/components/updates/ShowUpdate.vue";
import { mapGetters } from "vuex";
export default {
  components: {
    InfiniteLoading,
    ShowUpdate
  },
  data() {
    return {
      loading: true,
      error: false,

      offset: 0,
      limit: 20
    };
  },
  computed: {
    ...mapGetters(["updates"])
  },
  methods: {
    infiniteHandler($state) {
      this.$store
        .dispatch("loadUpdates", { offset: this.offset, limit: this.limit })
        .then(({ data }) => {
          if (data.length) {
            this.offset += this.limit;
            $state.loaded();
          } else {
            $state.complete();
          }
        });
    }
  }
};
</script>
