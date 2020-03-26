<template>
  <div>
    <v-list three-lines>
      <v-list-item v-for="(list, i) in lists" :key="i">
        <v-list-item-content>
          <v-list-item-title>
            <ChannelLink :channelId="list.id" :text="list.name" />
          </v-list-item-title>
          <v-list-item-subtitle>
            <router-link :to="`/list/${list.id}`" style="text-decoration:none; color: inherit">
              Contains {{list.channels.length}} channels
              <v-icon style="width: 13px" small>{{linkIcon}}</v-icon>
            </router-link>
          </v-list-item-subtitle>
        </v-list-item-content>
        <v-list-item-actions>
          <v-btn :to="`/subscribe/list/${list.id}`" color="primary" small>Subscribe</v-btn>
        </v-list-item-actions>
      </v-list-item>
    </v-list>
  </div>
</template>
<script>
import Api from "@/services/api.js";
import ChannelLink from "@/components/common/ChannelLink.vue";
import { mdiLaunch } from "@mdi/js";
export default {
  components: {
    ChannelLink
  },
  data() {
    return {
      linkIcon: mdiLaunch,

      loading: true,
      lists: null
    };
  },
  mounted() {
    Api()
      .get("lists/popular")
      .then(resp => {
        this.loading = false;
        this.lists = resp.data;
      });
  }
};
</script>