<template>
  <v-dialog :value="true" fullscreen hide-overlay>
    <v-card>
      <DialogToolbar
        actionText="Subscribe"
        v-on:closeDialog="$emit('closeDialog')"
        v-on:dialogAction="$emit('subscribe', channel, frequency)"
      />
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
import DialogToolbar from "@/components/common/DialogToolbar.vue";
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
export default {
  components: {
    ChannelIcon,
    DialogToolbar,
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
      frequency: {
        frequency: "Weekly",
        day: "Sat",
        time: "09:00:00"
      }
    };
  }
};
</script>