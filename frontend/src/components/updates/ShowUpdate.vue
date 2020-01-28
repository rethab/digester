<template>
  <div>
    <v-subheader style="height: 20px">
      <ChannelIcon class="mr-1" :small="true" :type="value.channelType" />
      <a
        :href="value.channelLink"
        target="_blank"
        style="text-decoration: none"
        class="mr-1"
        color="secondary"
      >{{value.channelName}}</a>
      |
      <span class="ml-1">{{ value.published | formatDate }}</span>
    </v-subheader>
    <v-divider></v-divider>
    <v-list-item class="mb-2" style="min-height: 25px">
      <v-list-item-content>
        <span>
          <a :href="value.url" target="_blank" class="black--text">{{linkText}}</a>
        </span>
      </v-list-item-content>
    </v-list-item>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import moment from "moment";
export default {
  components: {
    ChannelIcon
  },
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  computed: {
    linkText() {
      if (this.value.title) return this.value.title;
      else return this.value.url;
    }
  },
  filters: {
    formatDate(datetime) {
      return moment(datetime, "YYYY-MM-DDTHH:mm:ss").calendar(null, {
        lastWeek: "dddd",
        lastDay: "[Yesterday at] HH:mm",
        sameDay: "[Today at] HH:mm"
      });
    }
  }
};
</script>