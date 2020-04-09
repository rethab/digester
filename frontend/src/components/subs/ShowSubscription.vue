<template>
  <div>
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
        <v-card-text>
          <v-icon small>{{ calendarIcon }}</v-icon>
          {{value | showFrequency}}
        </v-card-text>
        <v-divider light class="mx-2"></v-divider>
        <v-card-actions>
          <v-spacer></v-spacer>
          <router-link :to="`/sub/${value.id}/edit`" text>
            <v-icon>{{ pencilIcon }}</v-icon>
          </router-link>
          <v-dialog v-model="deleteDialog" width="500">
            <template v-slot:activator="{ on }">
              <v-icon class="mr-1 ml-2" v-on="on" color="error lighten-1">{{ removeIcon }}</v-icon>
            </template>
            <v-card>
              <v-card-title>Are you sure?</v-card-title>
              <v-card-text>Please confirm that you want to delete this subscription.</v-card-text>
              <v-divider></v-divider>
              <v-card-actions>
                <v-btn text @click="deleteDialog = false">Cancel</v-btn>
                <v-spacer></v-spacer>
                <v-btn
                  @click="$emit('remove', value)"
                  :loading="deleteLoading"
                  color="error"
                  text
                >Confirm</v-btn>
              </v-card-actions>
            </v-card>
          </v-dialog>
        </v-card-actions>
      </div>
    </v-card>
  </div>
</template>

<script>
import ChannelIcon from "@/components/common/ChannelIcon.vue";
import ChannelLink from "@/components/common/ChannelLink.vue";
import { mdiCalendar, mdiPencilOutline, mdiDelete } from "@mdi/js";
export default {
  components: {
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
      calendarIcon: mdiCalendar,
      pencilIcon: mdiPencilOutline,
      removeIcon: mdiDelete,

      deleteLoading: false,
      deleteDialog: null
    };
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