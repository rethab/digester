<template>
  <v-dialog :value="true" fullscreen hide-overlay>
    <v-card>
      <v-toolbar dark color="primary">
        <v-btn icon dark @click="$emit('closeDialog')">
          <v-icon>{{ closeIcon }}</v-icon>
        </v-btn>
        <v-spacer></v-spacer>
        <v-toolbar-items>
          <v-btn dark text @click="$emit('subscribe', channel, frequency)">Subscribe</v-btn>
        </v-toolbar-items>
      </v-toolbar>
      <v-card-title>
        <span>
          <ChannelIcon :type="channel.channel_type" />
          {{channel.name}}
        </span>
      </v-card-title>
      <v-card-subtitle class="mt-3">How often do you want to receive digests?</v-card-subtitle>
      <v-card-text>
        <FrequencySelection v-model="frequency" />
      </v-card-text>
    </v-card>
  </v-dialog>
</template>
<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import { mdiClose } from "@mdi/js";
export default {
  components: {
    ChannelIcon,
    FrequencySelection
  },
  props: {
    channel: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      closeIcon: mdiClose,
      frequency: {
        frequency: "Weekly",
        day: "Sat",
        time: "09:00:00"
      }
    };
  }
};
</script>