<template>
  <v-card color="secondary" class="lighten-4">
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
      <v-card-subtitle v-if="!editing">
        <v-icon small>{{ calendarIcon }}</v-icon>
        {{value | showFrequency}}
        <v-icon @click="editing = true" small class="ml-3">{{ pencilIcon }}</v-icon>
      </v-card-subtitle>
      <v-card-subtitle v-else>
        <FrequencySelection v-model="value" />
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="editing = false" color="secondary" outlined>Cancel</v-btn>
          <v-btn @click.stop="save" class="primary">Save</v-btn>
        </v-card-actions>
      </v-card-subtitle>
    </div>
  </v-card>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import ChannelLink from "@/components/common/ChannelLink.vue";
import { mdiCalendar, mdiPencilOutline } from "@mdi/js";
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
  data() {
    return {
      editing: false,

      calendarIcon: mdiCalendar,
      pencilIcon: mdiPencilOutline
    };
  },
  computed: {
    channelLink() {
      if (!this.isList()) {
        return this.value.channelLink;
      } else {
        return `/list/${this.value.channelId}`;
      }
    }
  },
  methods: {
    isList() {
      return this.value.channelType == "List";
    },
    save() {
      this.$store
        .dispatch("updateSubscription", {
          id: this.value.id,
          frequency: this.value.frequency,
          day: this.value.day,
          time: this.value.time
        })
        .then(() => {
          this.editing = false;
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