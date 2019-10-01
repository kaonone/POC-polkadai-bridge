import React, { useState, useCallback } from 'react';
import { Grid } from '@material-ui/core';

import { Address } from '~components/Address';
import SendingForm, { SendingFormProps } from './SendingForm';

function SubstrateToEthereum() {
  const [selectedFromAddress, selectFromAddress] = useState<string | null>(null);

  const handleFormChange: NonNullable<SendingFormProps['onChange']> = useCallback(
    (values, errors) => {
      if (selectedFromAddress !== values.from) {
        !values.from && selectFromAddress(null);
        values.from && !errors.from && selectFromAddress(values.from);
      }
    },
    []
  );

  return (
    <Grid container spacing={2}>
      {selectedFromAddress && (
        <Grid item xs={12}>
          <Address type="substrate" address={selectedFromAddress} />
        </Grid>
      )}
      <Grid item xs={12}>
        <SendingForm onChange={handleFormChange} />
      </Grid>
    </Grid>
  );
}

export default SubstrateToEthereum;
