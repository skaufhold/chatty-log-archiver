CREATE TYPE message_flag AS ENUM ('moderator', 'prime', 'broadcaster', 'subscriber', 'staff');

CREATE TABLE public.messages
(
  id SERIAL NOT NULL,
  user_id INT NOT NULL,
  channel_id INT NOT NULL,
  message text NOT NULL,
  sent_at TIMESTAMP NOT NULL,
  flags message_flag[] NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT message_user_id_fk FOREIGN KEY (user_id) REFERENCES public.users (id) MATCH SIMPLE
  ON UPDATE CASCADE
  ON DELETE RESTRICT,
  CONSTRAINT message_channel_id_fk FOREIGN KEY (channel_id) REFERENCES public.channels (id) MATCH SIMPLE
  ON UPDATE CASCADE
  ON DELETE RESTRICT
);
