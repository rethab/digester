<template>
  <div>
    <v-row>
      <v-col class="text-center">
        <p class="title">You are in the Cockpit.</p>
        <p class="subtitle">Here you can create subscriptions, lists or update your settings.</p>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Subscribe"
          link="/subs"
          subtitle="to Lists or individual Channels"
          text="Receive weekly (or daily) E-mails with all new entries from blogs you have subscribed to."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Create Lists"
          link="/lists"
          subtitle="to bundle Blogs and Github Projects"
          text="Group Blogs about similar topics in Lists and share them with friends."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Updates"
          link="/updates"
          subtitle="from your subscriptions"
          text="This is for the impatient! If you can't wait to receive your digest via E-mail, this is where to look."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Settings"
          link="/settings"
          subtitle="for your Digester account"
          text="Are you travelling and need to update your timezone?"
        />
      </v-col>
    </v-row>
  </div>
</template>

<script>
import CockpitCard from "@/components/CockpitCard.vue";
export default {
  components: {
    CockpitCard
  }
};
</script>