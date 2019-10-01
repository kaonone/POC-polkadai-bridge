import * as React from 'react';
import { useCallback } from 'react';
import { Form, Field, FormSpy } from 'react-final-form';
import { FORM_ERROR } from 'final-form';
import { Button, Typography, Grid, Box } from '@material-ui/core';
import { O } from 'ts-toolbelt';

import { TextField } from '~components/form';
import { useApi } from '~components/context';
import { useSubscribable } from '~util/hooks';
import getErrorMsg from '~util/getErrorMsg';
import { validateFloat, validateRequired, validateSubstrateAddress } from '~util/validators';
import { DEFAULT_DECIMALS } from '~env';
import { Balance } from '~components/Balance';

interface FormData {
  address: string;
  amount: string;
}

const fields: { [key in keyof FormData]: key } = {
  address: 'address',
  amount: 'amount',
};

type Errors = Partial<O.Update<FormData, keyof FormData, string>>;

function validate(values: FormData): Errors {
  return {
    address: validateRequired(values.address) || validateSubstrateAddress(values.address),
    amount: validateRequired(values.amount) || validateFloat(values.amount, DEFAULT_DECIMALS),
  };
}

function SendingForm() {
  const api = useApi();
  const [account] = useSubscribable(() => api.getEthAccount$(), []);

  const handleSubmit = useCallback(async ({ address, amount }: FormData) => {
    try {
      if (!account) {
        throw new Error('Source account for token transfer not found');
      }
      await api.sendToSubstrate(account, address, amount);
    } catch (error) {
      return { [FORM_ERROR]: getErrorMsg(error) };
    }
  }, [account]);

  return (
    <Form<FormData>
      onSubmit={handleSubmit}
      subscription={{ submitting: true, submitError: true }}
      initialValues={{ address: '', amount: '' }}
      validate={validate}
    >
      {({ handleSubmit, submitting, submitError }): React.ReactElement<{}> => (
        <form onSubmit={handleSubmit}>
          <Grid container spacing={2}>
            <Grid item xs>
              <FormSpy<FormData> subscription={{ errors: true, values: true }}>
                {({ errors, values }: { values: FormData, errors: Errors }) => (
                  <Field
                    name={fields.address}
                    component={TextField}
                    fullWidth
                    variant="outlined"
                    label='Address'
                    margin="normal"
                    error={false}
                    InputLabelProps={{
                      shrink: true
                    }}
                    helperText={!errors.address && !!values.address && (
                      <Box color="primary.main">
                        Available: <Balance address={values.address} type="substrate" />
                      </Box>
                    )}
                    FormHelperTextProps={{
                      component: 'div',
                    }}
                  />
                )}
              </FormSpy>
            </Grid>
          </Grid>
          <Field
            name={fields.amount}
            component={TextField}
            fullWidth
            variant="outlined"
            label='Amount'
            margin="normal"
            error={false}
            InputLabelProps={{
              shrink: true
            }}
          />
          {!!submitError && <Typography variant='body1' color="error">{submitError}</Typography>}
          <Button fullWidth type="submit" variant="contained" color="primary" disabled={submitting}>
            Send{submitting && 'ing'}
          </Button>
        </form>
      )}
    </Form>
  );
}

export default SendingForm;
