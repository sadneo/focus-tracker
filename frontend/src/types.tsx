export interface EventObject {
  change_type: string;
  timestamp: {
    nanos_since_epoch: number;
    secs_since_epoch: number;
  };
  id: number;
  app_id: string;
}
