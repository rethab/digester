<template>
  <div>
    <v-card flat>
      <div>
        <v-card-title>
          <ChannelLink
            :channelId="value.channelId"
            :channelLink="value.channelLink"
            :text="value.name"
          />
          <ChannelIcon :type="value.channelType" class="pl-1" small />
        </v-card-title>
        <v-card-subtitle>
          <ChannelLink
            :channelId="value.channelId"
            :channelLink="value.channelLink"
            :text="value.channelLink ? value.channelLink : value.summary"
            class="grey--text"
          />
        </v-card-subtitle>
        <v-card-text>
          <FrequencySelection v-model="value" />
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn to="/subs" color="secondary" outlined>Cancel</v-btn>
            <v-btn @click.stop="save" class="primary">Save</v-btn>
          </v-card-actions>
        </v-card-text>
      </div>
    </v-card>
  </div>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import ChannelLink from "@/components/common/ChannelLink.vue";
export default {
  components: {
    FrequencySelection,
    ChannelIcon,
    ChannelLink
  },
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  methods: {
    save() {
      this.$store
        .dispatch("updateSubscription", {
          id: this.value.id,
          frequency: this.value.frequency,
          day: this.value.day,
          time: this.value.time
        })
        .then(() => {
          this.$router.push(`/subs`);
        });
    }
  },
  filters: {
    showFrequency: function(sub) {
      var fmt = "every ";
      if (sub.frequency == "Daily") fmt += "day";
      else fmt += sub.day;
      fmt += " at ";
      fmt += sub.time.substring(0, 5); // HH:MM
      return fmt;
    }
  }
};
</script>