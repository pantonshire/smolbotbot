from . import data

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker


db_connection_data = data.read_json("data/.db")

engine = create_engine(db_connection_data["uri"])

del db_connection_data

Session = sessionmaker(bind=engine)


def accessdb(create_transaction, *args):
    result = None
    session = Session()
    try:
        result = create_transaction(session, *args)
        session.commit()
    except:
        session.rollback()
        raise
    finally:
        session.close()
        session = None
    return result
