<template>
  <v-card color="secondary" class="lighten-4">
    <div>
      <v-card-title>{{value.name}}</v-card-title>
      <v-card-text>
        <span v-for="(channel, i) in value.channels" :key="channel.name">
          <span v-if="i > 0">|</span>
          <ChannelIcon :type="channel.channelType" :small="true" />
          {{channel.name}}
        </span>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn>Edit</v-btn>
        <v-btn>Delete</v-btn>
      </v-card-actions>
    </div>
  </v-card>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
export default {
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  components: {
    ChannelIcon
  },
  data() {
    return {};
  },
  computed: {}
};
</script>